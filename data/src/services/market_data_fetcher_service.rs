use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration as DurationChrono, Utc};
use reqwest::{Error, StatusCode};
use rust_decimal::Decimal;
use serde_json::Value;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use crate::lib::helper::Helper;
use crate::models::timeframe::{ContractType, TimeFrame};
use crate::{
    models::market_data::MarketData,
    repositories::{
        market_data_repository::MarketDataRepository, timeframe_repository::TimeFrameRepository,
    },
};

use super::database_service::DatabaseService;

const BINANCE_FUTURE_API_URL: &str = "https://fapi.binance.com/fapi/v1/";
const CONTINUOUS_KLINES_API_PATH: &str = "continuousKlines";
const MAX_RETRIES: i32 = 5;
const RATE_LIMIT_TIMEOUT: i64 = 100;
const RATE_LIMIT_MAX_WEIGHT: i32 = 4000;

#[derive(Debug)]
pub enum MarketDataFetcherError {
    Request(Error),
    Json(Error),
    Api { status: StatusCode, body: String },
}

impl fmt::Display for MarketDataFetcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketDataFetcherError::Request(e) => write!(f, "Request failed: {}", e),
            MarketDataFetcherError::Json(e) => write!(f, "Failed to parse JSON: {}", e),
            MarketDataFetcherError::Api { status, body } => {
                write!(f, "API error {}: {}", status, body)
            }
        }
    }
}

impl std::error::Error for MarketDataFetcherError {}

impl From<Error> for MarketDataFetcherError {
    fn from(err: Error) -> Self {
        MarketDataFetcherError::Request(err)
    }
}

pub struct MarketDataFetcher {
    pub client: reqwest::Client,
    pub symbol: String,
    pub contract_type: ContractType,
    pub timeframe: TimeFrame,
    pub lookback_days: u32,
    market_data_repository: Arc<MarketDataRepository>,
}

impl MarketDataFetcher {
    pub async fn new(
        symbol: String,
        contract_type: ContractType,
        interval: String,
        weight: Decimal,
        lookback_days: u32,
    ) -> Result<Self> {
        let timeframe_repository = TimeFrameRepository::new(DatabaseService::new().await?);
        let market_data_repository = MarketDataRepository::new(DatabaseService::new().await?);

        let timeframe = timeframe_repository
            .find_or_create(symbol.clone(), contract_type.clone(), interval, weight)
            .await?;

        Ok(MarketDataFetcher {
            client: reqwest::Client::new(),
            symbol,
            contract_type,
            timeframe,
            lookback_days,
            market_data_repository: Arc::new(market_data_repository),
        })
    }

    async fn fetch_with_retry(
        &self,
        path: &str,
        params: &[(&str, String)],
        retry_count: i32,
    ) -> Result<Value, MarketDataFetcherError> {
        let url = format!("{}{}", BINANCE_FUTURE_API_URL, path);
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(MarketDataFetcherError::Request)?;

        if let Some(weight) = response
            .headers()
            .get("x-mbx-used-weight-1m")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<i32>().ok())
        {
            if weight >= RATE_LIMIT_MAX_WEIGHT {
                tracing::warn!("Rate limit weight threshold reached: {}", weight);
                tokio::time::sleep(std::time::Duration::from_millis(RATE_LIMIT_TIMEOUT as u64))
                    .await;
                return Box::pin(self.fetch_with_retry(path, params, retry_count)).await;
            }
        }

        match response.status() {
            StatusCode::TOO_MANY_REQUESTS if retry_count < MAX_RETRIES => {
                tracing::warn!("Rate limited, retry {} of {}", retry_count + 1, MAX_RETRIES);
                tokio::time::sleep(std::time::Duration::from_millis(RATE_LIMIT_TIMEOUT as u64))
                    .await;
                Box::pin(self.fetch_with_retry(path, params, retry_count + 1)).await
            }
            _ => match response.error_for_status() {
                Ok(resp) => resp.json().await.map_err(MarketDataFetcherError::Json),
                Err(err) => {
                    let status = err.status().unwrap_or_default();
                    let body = err.to_string();
                    tracing::error!(%status, %body, "Binance API request failed");
                    Err(MarketDataFetcherError::Api { status, body })
                }
            },
        }
    }

    pub fn format_values_to_kline_create_payload(&self, value: Value) -> MarketData {
        let open_time = value[0].as_i64().unwrap();
        let open = value[1].as_str().unwrap();
        let high = value[2].as_str().unwrap();
        let low = value[3].as_str().unwrap();
        let close = value[4].as_str().unwrap();
        let volume = value[5].as_str().unwrap();
        let close_time = value[6].as_i64().unwrap();
        let trades = value[8].as_i64().unwrap();

        MarketData::new(
            self.timeframe.id,
            self.symbol.clone(),
            self.contract_type.to_string(),
            DateTime::from_timestamp_millis(open_time).unwrap(),
            DateTime::from_timestamp_millis(close_time).unwrap(),
            Decimal::from_str(open).unwrap_or_default(),
            Decimal::from_str(close).unwrap_or_default(),
            Decimal::from_str(high).unwrap_or_default(),
            Decimal::from_str(low).unwrap_or_default(),
            Decimal::from_str(volume).unwrap_or_default(),
            trades,
        )
    }

    pub async fn initialize_market_data(
        &self,
        limit: i16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Calculate the timestamp for one month ago
        let end_time = Utc::now();
        let start_time = end_time - DurationChrono::days(self.lookback_days.into());

        let mut current_time = start_time.timestamp_millis();
        while current_time < end_time.timestamp_millis() {
            let params = [
                ("pair", self.symbol.to_string()),
                ("contractType", self.contract_type.to_string()),
                (
                    "interval",
                    Helper::minutes_to_interval(self.timeframe.interval_minutes),
                ),
                ("startTime", current_time.to_string()),
                ("endTime", end_time.timestamp_millis().to_string()),
                ("limit", limit.to_string()),
            ];

            let data = self
                .fetch_with_retry(CONTINUOUS_KLINES_API_PATH, &params, MAX_RETRIES)
                .await?;

            let market_data_array = data.as_array().ok_or("Invalid response format")?;

            if market_data_array.is_empty() {
                break;
            }

            let market_data_batch: Vec<MarketData> = market_data_array
                .iter()
                .map(|raw_data| self.format_values_to_kline_create_payload(raw_data.clone()))
                .collect();

            self.market_data_repository
                .create_batch(&market_data_batch)
                .await?;

            if let Some(last_record) = market_data_batch.last() {
                current_time = last_record.open_time.timestamp_millis() + 1;
            }
        }

        Ok(())
    }

    pub async fn fetch_recent_market_data(
        &self,
        limit: i16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let latest_record = self
            .market_data_repository
            .find_latest_by_timeframe(
                &self.timeframe.id,
            )
            .await?;

        let start_time = match latest_record {
            Some(record) => record.open_time.timestamp_millis() + 1,
            None => (Utc::now() - DurationChrono::hours(24)).timestamp_millis(),
        };

        let end_time = Utc::now();
        let params = [
            ("pair", self.symbol.to_string()),
            ("contractType", self.contract_type.to_string()),
            (
                "interval",
                Helper::minutes_to_interval(self.timeframe.interval_minutes),
            ),
            ("startTime", start_time.to_string()),
            ("endTime", end_time.timestamp_millis().to_string()),
            ("limit", limit.to_string()),
        ];

        let data = self
            .fetch_with_retry(CONTINUOUS_KLINES_API_PATH, &params, MAX_RETRIES)
            .await?;
        let market_data_array = data.as_array().ok_or("Invalid response format")?;

        if !market_data_array.is_empty() {
            let market_data_batch: Vec<MarketData> = market_data_array
                .iter()
                .map(|raw_data| self.format_values_to_kline_create_payload(raw_data.clone()))
                .collect();

            self.market_data_repository
                .create_batch(&market_data_batch)
                .await?;
        }

        Ok(())
    }
}

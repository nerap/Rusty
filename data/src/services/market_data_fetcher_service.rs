use std::fmt;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use reqwest::StatusCode;
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;

use crate::lib::helper::Helper;
use crate::models::timeframe::TimeFrame;
use crate::{
    models::market_data::MarketData,
    repositories::{
        market_data_repository::MarketDataRepository, timeframe_repository::TimeFrameRepository,
    },
};

use super::database_service::DatabaseService;

const BINANCE_FUTURE_API_URL: &str = "https://fapi.binance.com/fapi/v1/";

const CONTINUOUS_KLINES_API_PATH: &str = "continuousKlines";

#[derive(Debug)]
pub enum MarketDataFetcherError {
    RequestError(reqwest::Error),
    JsonError(reqwest::Error),
    ApiError { status: StatusCode, body: String },
}

impl fmt::Display for MarketDataFetcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketDataFetcherError::RequestError(e) => write!(f, "Request failed: {}", e),
            MarketDataFetcherError::JsonError(e) => write!(f, "Failed to parse JSON: {}", e),
            MarketDataFetcherError::ApiError { status, body } => {
                write!(f, "API error {}: {}", status, body)
            }
        }
    }
}

impl std::error::Error for MarketDataFetcherError {}

impl From<reqwest::Error> for MarketDataFetcherError {
    fn from(err: reqwest::Error) -> Self {
        MarketDataFetcherError::RequestError(err)
    }
}

pub struct MarketDataFetcher {
    pub client: reqwest::Client,
    pub symbol: String,
    pub contract: String,
    pub timeframe: TimeFrame,
    timeframe_repository: Arc<TimeFrameRepository>,
    market_data_repository: Arc<MarketDataRepository>,
}

impl MarketDataFetcher {
    pub async fn new(
        symbol: String,
        contract: String,
        interval: String,
        weight: Decimal,
    ) -> Result<Self> {
        let timeframe_repository = TimeFrameRepository::new(DatabaseService::new().await?);
        let market_data_repository = MarketDataRepository::new(DatabaseService::new().await?);

        let timeframe = timeframe_repository
            .find_by_interval_and_weight(Helper::interval_to_minutes(&interval).unwrap(), weight)
            .await?
            .ok_or_else(|| anyhow!("TimeFrame not found"))?;

        Ok(MarketDataFetcher {
            client: reqwest::Client::new(),
            symbol,
            contract,
            timeframe,
            timeframe_repository: Arc::new(timeframe_repository),
            market_data_repository: Arc::new(market_data_repository),
        })
    }
    async fn fetch(
        &self,
        path: &str,
        params: &[(&str, String)],
    ) -> Result<Value, MarketDataFetcherError> {
        let url = format!("{}{}", BINANCE_FUTURE_API_URL, path);

        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await
            .map_err(MarketDataFetcherError::RequestError)?;

        println!("{:?}", response);
        match response.error_for_status() {
            Ok(resp) => resp.json().await.map_err(MarketDataFetcherError::JsonError),
            Err(err) => {
                let status = err.status().unwrap_or_default();
                let body = err.to_string();

                tracing::error!(%status, %body, "Binance API request failed");
                Err(MarketDataFetcherError::ApiError { status, body })
            }
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
            self.contract.clone(),
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
        limit: i8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Calculate the timestamp for one month ago
        let end_time = Utc::now();
        let start_time = end_time - Duration::days(30);

        let mut current_time = start_time.timestamp_millis();
        while current_time < end_time.timestamp_millis() {
            let params = [
                ("pair", self.symbol.to_string()),
                ("contractType", self.contract.to_string()),
                (
                    "interval",
                    Helper::minutes_to_interval(self.timeframe.interval_minutes),
                ),
                ("startTime", current_time.to_string()),
                ("endTime", end_time.timestamp_millis().to_string()),
                ("limit", limit.to_string()),
            ];

            let data = self.fetch(CONTINUOUS_KLINES_API_PATH, &params).await?;

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
}

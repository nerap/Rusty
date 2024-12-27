use anyhow::Result;
use chrono::{DateTime, Duration as DurationChrono, Utc};
use reqwest::{Error, StatusCode};
use rust_decimal::Decimal;
use serde_json::Value;
use std::str::FromStr;
use std::sync::Arc;
use std::{fmt, usize};
use tokio::time::sleep;

use crate::models::timeframe::{ContractType, TimeFrame};
use crate::utils::helper::Helper;
use crate::{
    models::market_data::MarketData,
    repositories::{
        market_data_repository::MarketDataRepository, timeframe_repository::TimeFrameRepository,
    },
};

use super::database_service::DatabaseService;

const BINANCE_FUTURE_API_URL: &str = "https://fapi.binance.com/fapi/v1/";
const CONTINUOUS_KLINES_API_PATH: &str = "continuousKlines";
const FETCH_LIMIT: i32 = 2;
const MAX_RETRIES: i32 = 5;
const RECENT_DATA_MAX_RETRIES: i32 = 3;
const RATE_LIMIT_TIMEOUT: i64 = 100;
const RECENT_DATA_RETRY_DELAY: u64 = 2000; // 2 seconds in milliseconds
const RATE_LIMIT_MAX_WEIGHT: i32 = 4000;

#[derive(Debug)]
pub enum MarketDataFetcherError {
    Request(Error),
    Json(Error),
    Api { status: StatusCode, body: String },
    NoDataFound,
}

impl fmt::Display for MarketDataFetcherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarketDataFetcherError::Request(e) => write!(f, "Request failed: {}", e),
            MarketDataFetcherError::Json(e) => write!(f, "Failed to parse JSON: {}", e),
            MarketDataFetcherError::Api { status, body } => {
                write!(f, "API error {}: {}", status, body)
            }
            MarketDataFetcherError::NoDataFound => write!(f, "No market data found"),
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
        lookback_days: u32,
    ) -> Result<Self> {
        let timeframe_repository = TimeFrameRepository::new(DatabaseService::new().await?);
        let market_data_repository = MarketDataRepository::new(DatabaseService::new().await?);

        let timeframe = timeframe_repository
            .find_or_create(symbol.clone(), contract_type.clone(), interval)
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
                sleep(std::time::Duration::from_millis(RATE_LIMIT_TIMEOUT as u64)).await;
                return Box::pin(self.fetch_with_retry(path, params, retry_count)).await;
            }
        }

        match response.status() {
            StatusCode::TOO_MANY_REQUESTS if retry_count < MAX_RETRIES => {
                tracing::warn!("Rate limited, retry {} of {}", retry_count + 1, MAX_RETRIES);
                sleep(std::time::Duration::from_millis(RATE_LIMIT_TIMEOUT as u64)).await;
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

    fn format_values_to_kline_create_payload(
        &self,
        value: Value,
    ) -> Result<MarketData, MarketDataFetcherError> {
        let open_time = value[0]
            .as_i64()
            .ok_or_else(|| MarketDataFetcherError::Api {
                status: StatusCode::BAD_REQUEST,
                body: "Invalid open_time format".to_string(),
            })?;

        let parse_decimal =
            |value: &Value, field: &str| -> Result<Decimal, MarketDataFetcherError> {
                value
                    .as_str()
                    .ok_or_else(|| MarketDataFetcherError::Api {
                        status: StatusCode::BAD_REQUEST,
                        body: format!("Invalid {} format", field),
                    })
                    .and_then(|s| {
                        Decimal::from_str(s).map_err(|_| MarketDataFetcherError::Api {
                            status: StatusCode::BAD_REQUEST,
                            body: format!("Invalid {} decimal", field),
                        })
                    })
            };

        Ok(MarketData::new(
            self.timeframe.id,
            self.symbol.clone(),
            self.contract_type.to_string(),
            DateTime::<Utc>::from_timestamp_millis(open_time).ok_or_else(|| {
                MarketDataFetcherError::Api {
                    status: StatusCode::BAD_REQUEST,
                    body: "Invalid timestamp".to_string(),
                }
            })?,
            DateTime::<Utc>::from_timestamp_millis(value[6].as_i64().ok_or_else(|| {
                MarketDataFetcherError::Api {
                    status: StatusCode::BAD_REQUEST,
                    body: "Invalid close_time format".to_string(),
                }
            })?)
            .ok_or_else(|| MarketDataFetcherError::Api {
                status: StatusCode::BAD_REQUEST,
                body: "Invalid timestamp".to_string(),
            })?,
            parse_decimal(&value[1], "open")?,
            parse_decimal(&value[4], "close")?,
            parse_decimal(&value[2], "high")?,
            parse_decimal(&value[3], "low")?,
            parse_decimal(&value[5], "volume")?,
            value[8]
                .as_i64()
                .ok_or_else(|| MarketDataFetcherError::Api {
                    status: StatusCode::BAD_REQUEST,
                    body: "Invalid trades format".to_string(),
                })?,
        ))
    }

    async fn fetch_market_data(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<usize, MarketDataFetcherError> {
        let mut inserted_count = 0;
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
                ("limit", FETCH_LIMIT.to_string()),
            ];

            let data = self
                .fetch_with_retry(CONTINUOUS_KLINES_API_PATH, &params, 0)
                .await?;
            let market_data_array = data.as_array().ok_or(MarketDataFetcherError::Api {
                status: StatusCode::BAD_REQUEST,
                body: "Invalid response format".to_string(),
            })?;

            if market_data_array.is_empty() {
                break;
            }

            let market_data_batch: Result<Vec<MarketData>, _> = market_data_array
                .iter()
                .map(|raw_data| self.format_values_to_kline_create_payload(raw_data.clone()))
                .collect();

            let market_data_batch = market_data_batch?;
            self.market_data_repository
                .create_batch(&market_data_batch)
                .await
                .map_err(|e| MarketDataFetcherError::Api {
                    status: StatusCode::INTERNAL_SERVER_ERROR,
                    body: e.to_string(),
                })?;

            if let Some(last_record) = market_data_batch.last() {
                current_time = last_record.open_time.timestamp_millis() + 1;
                inserted_count += market_data_batch.len();
            }
        }

        if inserted_count == 0 {
            return Err(MarketDataFetcherError::NoDataFound);
        }

        Ok(inserted_count)
    }

    pub async fn initialize_market_data(&self) -> Result<usize, MarketDataFetcherError> {
        let end_time = Utc::now();
        let start_time = end_time - DurationChrono::days(self.lookback_days.into());

        self.fetch_market_data(start_time, end_time).await
    }

    pub async fn fetch_recent_market_data(&self) -> Result<usize, MarketDataFetcherError> {
        let latest_record = self
            .market_data_repository
            .find_latest_by_timeframe(&self.timeframe.id)
            .await
            .map_err(|e| MarketDataFetcherError::Api {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                body: e.to_string(),
            })?;

        let start_time = match latest_record {
            Some(record) => record.open_time + DurationChrono::milliseconds(1),
            None => Utc::now() - DurationChrono::hours(24),
        };

        let end_time = Utc::now();
        let mut retries = 0;

        loop {
            match self.fetch_market_data(start_time, end_time).await {
                Ok(count) => return Ok(count),
                Err(MarketDataFetcherError::NoDataFound) if retries < RECENT_DATA_MAX_RETRIES => {
                    retries += 1;
                    tracing::warn!(
                        "No recent data found, retry {} of {}",
                        retries,
                        RECENT_DATA_MAX_RETRIES
                    );
                    sleep(std::time::Duration::from_millis(RECENT_DATA_RETRY_DELAY)).await;
                }
                Err(e) => return Err(e),
            }
        }
    }
}

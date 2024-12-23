use std::fmt::{Debug, Result};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use dotenvy::Error;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

use crate::models::kline::KlineCreatePayload;
use crate::services::database_service::DatabaseService; // This imports the `Error` trait, not the `std::io::Error` struct

#[derive(Debug, Serialize, Deserialize)]
pub enum Position {
    Long,
    Short,
    Neutral,
}

const BINANCE_FUTURE_API_URL: &str = "https://fapi.binance.com/fapi/v1/";

const CONTINUOUS_KLINES_API_PATH: &str = "continuousKlines";

pub struct KlineFetcher {
    symbol: String,
    interval: String,
    contract: String,
}

impl KlineFetcher {
    pub fn new(symbol: String, interval: String, contract: String) -> Self {
        KlineFetcher {
            symbol,
            interval,
            contract,
        }
    }
    pub async fn fetch(&self, path: String, params: [(&str, &str)]) -> Result<Value, Error> {
        let client = Client::new();

        let url = format!("{}{}", BINANCE_FUTURE_API_URL, path,);

        // Send the HTTP GET request

        let response = client.get(&url).query(params).send().await;

        // Ensure the response status is OK
        if !response.status().is_success() {
            eprintln!(
                "Failed to fetch data. HTTP Status: {}, {:?}",
                response.status(),
                response
            );
            return Error(response.err());
        } else {
            eprintln!(
                "Failed to fetch data. HTTP Status: {}, {:?}",
                response.status(),
                response
            );
        }

        Ok(())
    }

    pub async fn fetchContinousKlines(
        &self,
        days: i8,
        limit: i8,
    ) -> Result<(), Box<dyn std::error::Error>> {
    }

    pub async fn fetchI() -> Result<(), Box<dyn std::error::Error>> {
        // Calculate the timestamp for one month ago
        let end_time = Utc::now().timestamp_millis();
        let start_time = (Utc::now() - Duration::days(60)).timestamp_millis();

        let mut current_start_time = start_time;

        println!("Historical data for {}:", symbol);
        //        let db = DatabaseService::new().await?;

        let params = [
            ("symbol", self.symbol),
            ("contract", self.contract),
            ("interval", self.interval),
            ("current_start_time", current_start_time),
            ("end_time", end_time),
            ("limit", limit),
        ];

        // Loop to fetch data in batches
        while current_start_time < end_time {
            let url = format!(
            "https://fapi.binance.com/fapi/v1/continuousKlines?pair={}&interval={}&limit={}&startTime={}&endTime={}&contractType={}",
            symbol, interval, limit, current_start_time, end_time, contract
        );

            // Send the HTTP GET request
            let response = reqwest::get(&url).await?;

            // Ensure the response status is OK
            if response.status().is_success() {
                // Parse the response as JSON
                let data: Value = response.json().await?;

                // Check if data is empty
                if data.as_array().unwrap().is_empty() {
                    break;
                }

                // Print the historical data
                for kline in data.as_array().unwrap() {
                    let open_time = kline[0].as_i64().unwrap();
                    let open_price = kline[1].as_str().unwrap();
                    let high_price = kline[2].as_str().unwrap();
                    let low_price = kline[3].as_str().unwrap();
                    let close_price = kline[4].as_str().unwrap();
                    let volume = kline[5].as_str().unwrap();
                    let base_asset_volume = kline[7].as_str().unwrap();
                    let number_of_trade = kline[8].as_i64().unwrap();
                    let taker_buy_volume = kline[9].as_str().unwrap();
                    let taker_buy_base_asser_volume = kline[10].as_str().unwrap();

                    // Extract and parse the data
                    // Parse decimal values
                    let kline_values = KlineCreatePayload {
                        symbol: "BTCUSDT".to_string(),
                        contract_type: "PERPETUAL".to_string(),
                        open_time: DateTime::from_timestamp_millis(open_time).unwrap(),
                        open_price: Decimal::from_str(open_price).unwrap_or_default(),
                        high_price: Decimal::from_str(high_price).unwrap_or_default(),
                        low_price: Decimal::from_str(low_price).unwrap_or_default(),
                        close_price: Decimal::from_str(close_price).unwrap_or_default(),
                        volume: Decimal::from_str(volume).unwrap_or_default(),
                        base_asset_volume: Decimal::from_str(base_asset_volume).unwrap_or_default(),
                        number_of_trades: number_of_trade,
                        taker_buy_volume: Decimal::from_str(taker_buy_volume).unwrap_or_default(),
                        taker_buy_base_asset_volume: Decimal::from_str(taker_buy_base_asser_volume)
                            .unwrap_or_default(),
                    };

                    // let row = db.kline.create(kline_values).await;
                    // println!("{:?}", row);

                    let open_time_date = NaiveDateTime::from_timestamp_millis(open_time)
                        .unwrap()
                        .format("%Y-%m-%d %H:%M:%S");

                    println!(
                        "Time: {}, Open: {}, High: {}, Low: {}, Close: {}, Volume: {}",
                        open_time_date, open_price, high_price, low_price, close_price, volume
                    );
                    // Update the current_start_time to the next timestamp
                    current_start_time = open_time + 1;
                }
            } else {
                eprintln!(
                    "Failed to fetch data. HTTP Status: {}, {:?}",
                    response.status(),
                    response
                );
                break;
            }
        }

        Ok(())
    }
}

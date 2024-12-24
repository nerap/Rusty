use std::fmt::{Debug, Result};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use reqwest::{Client, Error};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;


const BINANCE_FUTURE_API_URL: &str = "https://fapi.binance.com/fapi/v1/";

const CONTINUOUS_KLINES_API_PATH: &str = "continuousKlines";

pub struct MarketDataFetcher {
    pub client: Client,
    pub symbol: String,
    pub interval: String,
    pub contract: String,
}

impl MarketDataFetcher {
    pub fn new(symbol: String, interval: String, contract: String) -> Self {
        MarketDataFetcher {
            client: Client::new(),
            symbol,
            interval,
            contract,
        }
    }
    async fn fetch(&self, path: &str, params: &[(&str, String)]) -> Result<Value, reqwest::Error> {
        let url = format!("{}{}", BINANCE_FUTURE_API_URL, path);

        // Send the HTTP GET request
        let response = self.client.get(&url).query(params).send().await?;

        if response.status().is_success() {
            // Parse and return JSON response
            response.json().await
        } else {
            // Log the error details and return the error
            let status = response.status();
            let text = response.text().await?;
            eprintln!("HTTP request failed. Status: {}, Body: {}", status, text);

            Err(reqwest::Error::from(status))
        }
    }

    pub fn format_values_to_kline_create_payload(&self, value: Value) -> KlineCreatePayload {
        let open_time = value[0].as_i64().unwrap();
        let open_price = value[1].as_str().unwrap();
        let high_price = value[2].as_str().unwrap();
        let low_price = value[3].as_str().unwrap();
        let close_price = value[4].as_str().unwrap();
        let volume = value[5].as_str().unwrap();
        let close_time = value[6].as_i64().unwrap();
        let base_asset_volume = value[7].as_str().unwrap();
        let number_of_trade = value[8].as_i64().unwrap();
        let taker_buy_volume = value[9].as_str().unwrap();
        let taker_buy_base_asser_volume = value[10].as_str().unwrap();


        MarketData::new(timeframe_id, symbol, contract_type, open_time, close_time, open, high, low, close, volume, trades, vwap)
        // Parse decimal values
        // KlineCreatePayload {
        //     symbol: self.symbol.clone(),
        //     contract_type: self.contract.clone(),
        //     open_time: DateTime::from_timestamp_millis(open_time).unwrap(),
        //     close_time: DateTime::from_timestamp_millis(close_time).unwrap(),
        //     open_price: Decimal::from_str(open_price).unwrap_or_default(),
        //     high_price: Decimal::from_str(high_price).unwrap_or_default(),
        //     low_price: Decimal::from_str(low_price).unwrap_or_default(),
        //     close_price: Decimal::from_str(close_price).unwrap_or_default(),
        //     volume: Decimal::from_str(volume).unwrap_or_default(),
        //     base_asset_volume: Decimal::from_str(base_asset_volume).unwrap_or_default(),
        //     number_of_trades: number_of_trade,
        //     taker_buy_volume: Decimal::from_str(taker_buy_volume).unwrap_or_default(),
        //     taker_buy_base_asset_volume: Decimal::from_str(taker_buy_base_asser_volume)
        //         .unwrap_or_default(),
        // }
    }

    pub async fn fetchI(&self, limit: i8) -> Result<(), Box<dyn std::error::Error>> {
        // Calculate the timestamp for one month ago
        let end_time = Utc::now().timestamp_millis();
        let start_time = (Utc::now() - Duration::days(60)).timestamp_millis();

        let mut current_start_time = start_time;

        //        let db = DatabaseService::new().await?;

        let params = [
            ("symbol", self.symbol.to_string()),
            ("contract", self.contract.to_string()),
            ("interval", self.interval.to_string()),
            ("current_start_time", current_start_time.to_string()),
            ("end_time", end_time.to_string()),
            ("limit", limit.to_string()),
        ];

        // Loop to fetch data in batches
        while current_start_time < end_time {
            let data = self.fetch(CONTINUOUS_KLINES_API_PATH, params).await;

            // Check if data is empty
            if data.as_array().unwrap().is_empty() {
                break;
            }

            // Print the historical data
            for kline in data.as_array().unwrap() {
                let kline_create_payload = self.format_values_to_kline_create_payload(kline);

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
        }

        Ok(())
    }
}

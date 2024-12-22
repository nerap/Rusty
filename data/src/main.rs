// use std::collections::HashMap;
//
//

use chrono::{Duration, NaiveDateTime, Utc};
use reqwest::Error;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Define the Binance API endpoint for historical data
    let symbol = "BTCUSDT"; // Trading pair
    let interval = "1m"; // Time interval, e.g., 1m, 1h, 1d
    let contract = "PERPETUAL"; // Time interval, e.g., 1m, 1h, 1d
    let limit = 1; // Maximum number of data points per API call

    // Calculate the timestamp for one month ago
    let end_time = Utc::now().timestamp_millis();
    let start_time = (Utc::now() - Duration::days(30)).timestamp_millis();

    let mut current_start_time = start_time;

    println!("Historical data for {}:", symbol);

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
                println!("{:?}", kline);
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

                // Convert timestamp to human-readable date
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

            eprintln!("Failed to fetch data. HTTP Status: {}, {:?}", response.status(), response);
            break;
        }
    }

    Ok(())
}

use rust_decimal::Decimal;
use services::market_data_fetcher_service::MarketDataFetcher;

// src/main.rs
mod models;
mod repositories;
mod services;
mod lib;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let m = MarketDataFetcher::new(
        "BTCUSDT".to_string(),
        "PERPETUAL".to_string(),
        "15m".to_string(),
        Decimal::new(4, 1),
    )
    .await?;
    m.initialize_market_data(100).await;
    Ok(())
}

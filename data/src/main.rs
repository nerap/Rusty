use dotenvy::dotenv;
use models::timeframe;
use repositories::market_data_repository::MarketDataRepositoryError;
use rust_decimal::Decimal;
use services::{
    configuration_service::ConfigService, market_data_analyzer_service::MarketDataAnalyzer,
    market_data_fetcher_service::MarketDataFetcher,
};
use std::{env, path::Path};

// src/main.rs
mod lib;
mod models;
mod repositories;
mod services;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv()?;

    env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

    let config_str = std::fs::read_to_string("configuration.yaml")?;
    let config = ConfigService::load_config(&config_str)?.data;

    println!("Loaded config: {:#?}", config);

    for pair in config.pairs {
        for timeframe in pair.timeframes {

        println!("ehre");
            let market_data_fetcher = MarketDataFetcher::new(
                pair.symbol.to_string(),
                pair.contract_type.clone(),
                timeframe.interval.to_string(),
                timeframe.weight,
                config.lookback_days,
            )
            .await?;
            let result = market_data_fetcher.initialize_market_data(500).await;
            println!("{:?}", result);
        }
    }
    // let analyzer = MarketDataAnalyzer::new().await?;
    //
    // analyzer.analyze_market_data().await;

    Ok(())
}

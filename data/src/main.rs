use dotenvy::dotenv;
use models::timeframe::ContractType;
use services::{
    configuration_service::ConfigService,
    market_data_analyzer_service::MarketDataAnalyzer,
    market_data_fetcher_service::MarketDataFetcher,
};
use thiserror::Error;
use std::{env, sync::Arc, time::Duration};
use tokio::{sync::Semaphore, time::sleep};
use anyhow::Result;

mod models;
mod repositories;
mod services;
mod utils;

const DEFAULT_SLEEP: u64 = 10;
const MAX_CONCURRENT_TASKS: usize = 5;

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Market data error: {0}")]
    MarketData(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

impl std::convert::From<Box<dyn std::error::Error>> for WorkerError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        WorkerError::MarketData(error.to_string())
    }
}

async fn run_timeframe_worker(
    symbol: String,
    contract_type: ContractType,
    interval: String,
    lookback_days: u32,
    semaphore: Arc<Semaphore>,
) -> Result<(), WorkerError> {
    let _permit = semaphore.acquire().await.map_err(|e| WorkerError::Config(e.to_string()))?;

    let market_data_fetcher = MarketDataFetcher::new(
        symbol,
        contract_type,
        interval,
        lookback_days,
    ).await.map_err(|e| WorkerError::MarketData(e.to_string()))?;

    market_data_fetcher.initialize_market_data().await
        .map_err(|e| WorkerError::MarketData(e.to_string()))?;
    let analyzer = MarketDataAnalyzer::new().await
        .map_err(|e| WorkerError::Database(e.to_string()))?;
    analyzer.analyze_market_data().await
        .map_err(|e| WorkerError::Database(e.to_string()))?;

    loop {
        if let Err(e) = market_data_fetcher.fetch_recent_market_data().await {
            eprintln!("Error fetching market data: {}", e);
            continue;
        }

        match MarketDataAnalyzer::new().await {
            Ok(analyzer) => {
                if let Err(e) = analyzer.analyze_market_data().await {
                    eprintln!("Error analyzing market data: {}", e);
                }
            },
            Err(e) => eprintln!("Error creating analyzer: {}", e),
        }

        sleep(Duration::from_secs(DEFAULT_SLEEP)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkerError> {
dotenv().map_err(|e| WorkerError::Config(e.to_string()))?;
    env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

    let config_str = std::fs::read_to_string("configuration.yaml")
        .map_err(|e| WorkerError::Config(e.to_string()))?;
    let config = ConfigService::load_config(&config_str)
        .map_err(|e| WorkerError::Config(e.to_string()))?.data;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));
    let mut handles = vec![];

    for pair in config.pairs {
        for timeframe in pair.timeframes {
            let sem = Arc::clone(&semaphore);
            let handle = tokio::spawn(run_timeframe_worker(
                pair.symbol.to_string(),
                pair.contract_type.clone(),
                timeframe.interval.to_string(),
                config.lookback_days,
                sem,
            ));
            handles.push(handle);
        }
    }

    for handle in handles {
        if let Err(e) = handle.await {
            eprintln!("Task failed: {}", e);
        }
    }

    Ok(())
}

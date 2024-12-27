use anyhow::Result;
use dotenvy::dotenv;
use models::timeframe::{ContractType, Interval};
use services::{
    configuration_service::ConfigService,
    market_data_analyzer_service::MarketDataAnalyzer,
    market_data_fetcher_service::MarketDataFetcher,
};
use std::{env, str::FromStr, sync::Arc};
use tokio::sync::Semaphore;
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::helper::WorkerError;

mod models;
mod repositories;
mod services;
mod utils;

const MAX_CONCURRENT_TASKS: usize = 5;

fn get_cron_expression(interval: &str) -> String {
    match Interval::from_str(interval).unwrap() {
        Interval::Minute1 => "0 * * * * *",         // Every minute
        Interval::Minute3 => "0 */3 * * * *",       // Every 3 minutes
        Interval::Minute5 => "0 */5 * * * *",       // Every 5 minutes
        Interval::Minute15 => "0 */15 * * * *",     // Every 15 minutes
        Interval::Minute30 => "0 */30 * * * *",     // Every 30 minutes
        Interval::Hour1 => "0 0 * * * *",          // Every hour
        Interval::Hour2 => "0 0 */2 * * *",        // Every 2 hours
        Interval::Hour4 => "0 0 */4 * * *",        // Every 4 hours
        Interval::Hour6 => "0 0 */6 * * *",        // Every 6 hours
        Interval::Hour8 => "0 0 */8 * * *",        // Every 8 hours
        Interval::Hour12 => "0 0 */12 * * *",      // Every 12 hours
        Interval::Day1 => "0 0 0 * * *",           // Every day at midnight
        Interval::Day3 => "0 0 0 */3 * *",         // Every 3 days
        Interval::Week1 => "0 0 0 * * 0",          // Every Sunday at midnight
    }.to_string()
}

async fn run_timeframe_worker(
    symbol: String,
    contract_type: ContractType,
    interval: String,
    lookback_days: u32,
    semaphore: Arc<Semaphore>,
) -> Result<(), WorkerError> {
    let scheduler = JobScheduler::new().await
        .map_err(|e| WorkerError::Config(e.to_string()))?;

    let market_data_fetcher = Arc::new(
        MarketDataFetcher::new(symbol, contract_type, interval.clone(), lookback_days)
            .await
            .map_err(|e| WorkerError::MarketData(e.to_string()))?
    );

    // Initial data fetch
    market_data_fetcher
        .initialize_market_data()
        .await
        .map_err(|e| WorkerError::MarketData(e.to_string()))?;

    let cron_expression = get_cron_expression(&interval);
    let sem = Arc::clone(&semaphore);
    let fetcher = Arc::clone(&market_data_fetcher);

    let job = Job::new_async(cron_expression.as_str(), move |_uuid, _lock| {
        let sem = Arc::clone(&sem);
        let fetcher = Arc::clone(&fetcher);

        Box::pin(async move {
            let _permit = match sem.acquire().await {
                Ok(permit) => permit,
                Err(e) => {
                    eprintln!("Error acquiring semaphore: {}", e);
                    return;
                }
            };

            if let Err(e) = fetcher.fetch_recent_market_data().await {
                eprintln!("Error fetching market data: {}", e);
                return;
            }
            println!("success fetch");

            match MarketDataAnalyzer::new().await {
                Ok(analyzer) => {
                    if let Err(e) = analyzer.analyze_market_data().await {
                        eprintln!("Error analyzing market data: {}", e);
                    }
                }
                Err(e) => eprintln!("Error creating analyzer: {}", e),
            }
        })
    })
    .map_err(|e| WorkerError::Config(e.to_string()))?;

    scheduler
        .add(job)
        .await
        .map_err(|e| WorkerError::Config(e.to_string()))?;

    scheduler
        .start()
        .await
        .map_err(|e| WorkerError::Config(e.to_string()))?;

    // Keep the task running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await;
    }
}

#[tokio::main]
async fn main() -> Result<(), WorkerError> {
    dotenv().map_err(|e| WorkerError::Config(e.to_string()))?;
    env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");

    let config_str = std::fs::read_to_string("configuration.yaml")
        .map_err(|e| WorkerError::Config(e.to_string()))?;
    let config = ConfigService::load_config(&config_str)
        .map_err(|e| WorkerError::Config(e.to_string()))?
        .data;

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

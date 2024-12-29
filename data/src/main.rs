use anyhow::Result;
use clap::Parser;
use dotenvy::dotenv;
use models::timeframe::{ContractType, Interval};
use services::{
    configuration_service::ConfigService, market_data_analyzer_service::MarketDataAnalyzer,
    market_data_fetcher_service::MarketDataFetcher,
};
use std::{ path::Path, str::FromStr, sync::Arc};
use tokio::sync::broadcast;
use tokio::sync::Semaphore;
use tokio_cron_scheduler::{Job, JobScheduler};
use utils::helper::WorkerError;

mod models;
mod repositories;
mod services;
mod utils;

#[derive(Parser)]
#[command(name = "greet")]
#[command(author = "Your Name")]
#[command(version = "1.0")]
#[command(about = "A friendly greeting CLI", long_about = None)]
struct Args {
    #[arg(short = 'c', long = "config")]
    configuration: String,

    #[arg(short = 'i', long = "init", default_value_t = true, action = clap::ArgAction::Set)]
    initialize: bool,
}

fn setup_logging() {
    tracing_subscriber::fmt()
        .with_env_filter("info") // or "debug", "trace" etc
        .init();
}

const MAX_CONCURRENT_TASKS: usize = 5;

fn get_cron_expression(interval: &str) -> String {
    match Interval::from_str(interval).unwrap() {
        Interval::Minute1 => "0 * * * * *",     // Every minute
        Interval::Minute3 => "0 */3 * * * *",   // Every 3 minutes
        Interval::Minute5 => "0 */5 * * * *",   // Every 5 minutes
        Interval::Minute15 => "0 */15 * * * *", // Every 15 minutes
        Interval::Minute30 => "0 */30 * * * *", // Every 30 minutes
        Interval::Hour1 => "0 0 * * * *",       // Every hour
        Interval::Hour2 => "0 0 */2 * * *",     // Every 2 hours
        Interval::Hour4 => "0 0 */4 * * *",     // Every 4 hours
        Interval::Hour6 => "0 0 */6 * * *",     // Every 6 hours
        Interval::Hour8 => "0 0 */8 * * *",     // Every 8 hours
        Interval::Hour12 => "0 0 */12 * * *",   // Every 12 hours
        Interval::Day1 => "0 0 0 * * *",        // Every day at midnight
        Interval::Day3 => "0 0 0 */3 * *",      // Every 3 days
        Interval::Week1 => "0 0 0 * * 0",       // Every Sunday at midnight
    }
    .to_string()
}

async fn run_timeframe_worker(
    symbol: String,
    contract_type: ContractType,
    interval: String,
    lookback_days: u32,
    semaphore: Arc<Semaphore>,
    initialize: bool,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<(), WorkerError> {
    let mut scheduler = JobScheduler::new()
        .await
        .map_err(|e| WorkerError::Config(e.to_string()))?;

    let market_data_fetcher = Arc::new(
        MarketDataFetcher::new(
            symbol.clone(),
            contract_type.clone(),
            interval.clone(),
            lookback_days,
        )
        .await
        .map_err(|e| WorkerError::MarketData(e.to_string()))?,
    );

    if initialize {
        // Initial data fetch
        market_data_fetcher
            .initialize_market_data()
            .await
            .map_err(|e| WorkerError::MarketData(e.to_string()))?;

        // Analyze MarketData
        match MarketDataAnalyzer::new().await {
            Ok(analyzer) => {
                if let Err(e) = analyzer.analyze_market_data().await {
                    eprintln!("Error analyzing market data: {}", e);
                }
            }
            Err(e) => eprintln!("Error creating analyzer: {}", e),
        }
    }

    let cron_expression = get_cron_expression(&interval);
    let sem = Arc::clone(&semaphore);
    let fetcher = Arc::clone(&market_data_fetcher);

    let job = Job::new_async(cron_expression.as_str(), move |_uuid, _lock| {
        let sem = Arc::clone(&sem);
        let fetcher = Arc::clone(&fetcher);

        tracing::info!(
            "Running Job {} {} {}",
            symbol.clone(),
            interval.clone(),
            contract_type.clone()
        );

        Box::pin(async move {
            let _permit = match sem.acquire().await {
                Ok(permit) => permit,
                Err(e) => {
                    eprintln!("Error acquiring semaphore: {}", e);
                    return;
                }
            };

            // Fetch recent market data
            if let Err(e) = fetcher.fetch_recent_market_data().await {
                eprintln!("Error fetching market data: {}", e);
                return;
            }

            // Analyze MarketData
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

    match shutdown.recv().await {
        Ok(_) | Err(_) => {
            scheduler
                .shutdown()
                .await
                .map_err(|e| WorkerError::Config(e.to_string()))?
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), WorkerError> {
    setup_logging();

    let args = Args::parse();
    let _ = dotenv();
    let (shutdown_sender, _) = broadcast::channel(1);

    let config_str =
        std::fs::read_to_string(Path::new(&args.configuration).canonicalize().unwrap())
            .map_err(|e| WorkerError::Config(e.to_string()))?;

    let config = ConfigService::load_config(&config_str)
        .map_err(|e| WorkerError::Config(e.to_string()))?
        .data;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_TASKS));
    let mut handles = vec![];

    for pair in config.pairs {
        for timeframe in pair.timeframes {
            let sem = Arc::clone(&semaphore);
            let shutdown_rx = shutdown_sender.subscribe();

            let handle = tokio::spawn(run_timeframe_worker(
                pair.symbol.clone(),
                pair.contract_type.clone(),
                timeframe.interval.to_string(),
                config.lookback_days,
                sem,
                args.initialize,
                shutdown_rx,
            ));
            handles.push(handle);
        }
    }

    // Wait for either Ctrl+C or all workers to complete
    tokio::select! {
        _ = async {
            tokio::signal::ctrl_c().await.expect("failed to listen for ctrl-c");
            tracing::info!("Received shutdown signal, stopping all workers...");
            let _ = shutdown_sender.send(());
        } => {},
        _ = async {
            for handle in handles {
                if let Err(e) = handle.await {
                    tracing::error!("Task failed: {}", e);
                }
            }
        } => {}
    }

    tracing::info!("Shutdown complete");

    Ok(())
}

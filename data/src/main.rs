use services::kline_fetcher_service::KlineFetcher;

// src/main.rs
mod models;
mod repositories;
mod services;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    KlineFetcher::fetch().await;
    Ok(())
}

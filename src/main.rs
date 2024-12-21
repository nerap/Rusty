mod neural_network;
mod trading;

use trading::TradingBot;

#[tokio::main]
async fn main() {
    println!("Neural Trading Bot initialized!");

    // Initialize the trading bot with a 5-4-1 architecture
    let mut bot = TradingBot::new(&[5, 4, 1]);

    // Example usage
    let market_data = vec![0.5, 0.3, 0.7, 0.2, 0.4];
    let prediction = bot.predict(&market_data);

    println!("Market prediction: {}", prediction);
}

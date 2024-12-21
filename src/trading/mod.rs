pub mod data;

use crate::neural_network::NeuralNetwork;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TradePosition {
    pub entry_price: f64,
    pub position_size: f64,
    pub entry_time: DateTime<Utc>,
    pub trade_type: TradeType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TradeType {
    Long,
    Short,
}

pub struct TradingBot {
    network: NeuralNetwork,
    positions: Vec<TradePosition>,
}

impl TradingBot {
    pub fn new(layer_sizes: &[usize]) -> Self {
        TradingBot {
            network: NeuralNetwork::new(layer_sizes, 0.1),
            positions: Vec::new(),
        }
    }

    pub fn predict(&mut self, market_data: &[f64]) -> f64 {
        let prediction = self.network.forward(market_data);
        prediction[0]
    }

    pub fn train(&mut self, inputs: &[f64], targets: &[f64]) {
        self.network.train(inputs, targets);
    }
}

use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketData {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

impl MarketData {
    pub fn to_features(&self) -> Vec<f64> {
        vec![
            self.open,
            self.high,
            self.low,
            self.close,
            self.volume,
        ]
    }
}

pub fn normalize_data(data: &[f64]) -> Vec<f64> {
    if data.is_empty() {
        return vec![];
    }

    let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));

    data.iter()
        .map(|&x| (x - min) / (max - min))
        .collect()
}

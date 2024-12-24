// models/market_data.rs
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct MarketData {
    pub id: Uuid,
    pub timeframe_id: Uuid,

    #[validate(length(min = 1, max = 20))]
    pub symbol: String,

    #[validate(length(min = 1, max = 10))]
    pub contract_type: String,

    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open: Decimal,
    pub close: Decimal,
    pub high: Decimal,
    pub low: Decimal,
    pub volume: Decimal,
    pub trades: i64,

    // Technical indicators
    pub rsi_14: Option<Decimal>,
    pub macd_line: Option<Decimal>,
    pub macd_signal: Option<Decimal>,
    pub macd_histogram: Option<Decimal>,
    pub bb_upper: Option<Decimal>,
    pub bb_middle: Option<Decimal>,
    pub bb_lower: Option<Decimal>,
    pub atr_14: Option<Decimal>,

    // Market microstructure
    pub bid_ask_spread: Option<Decimal>,
    pub depth_imbalance: Option<Decimal>,
    pub funding_rate: Option<Decimal>,
    pub open_interest: Option<Decimal>,
    pub long_short_ratio: Option<Decimal>,

    // Volatility metrics
    pub volatility_1h: Option<Decimal>,
    pub volatility_24h: Option<Decimal>,

    // Price changes
    pub price_change_1h: Option<Decimal>,
    pub price_change_24h: Option<Decimal>,

    // Trading volume changes
    pub volume_change_1h: Option<Decimal>,
    pub volume_change_24h: Option<Decimal>,

    // Analyzed
    pub analyzed: bool,

    // Usable by model
    pub usable_by_model: bool,

    pub created_at: DateTime<Utc>,
}

impl MarketData {
    pub fn new(
        timeframe_id: Uuid,
        symbol: String,
        contract_type: String,
        open_time: DateTime<Utc>,
        close_time: DateTime<Utc>,
        open: Decimal,
        close: Decimal,
        high: Decimal,
        low: Decimal,
        volume: Decimal,
        trades: i64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timeframe_id,
            symbol,
            contract_type,
            open_time,
            close_time,
            open,
            high,
            low,
            close,
            volume,
            trades,
            rsi_14: None,
            macd_line: None,
            macd_signal: None,
            macd_histogram: None,
            bb_upper: None,
            bb_middle: None,
            bb_lower: None,
            atr_14: None,
            bid_ask_spread: None,
            depth_imbalance: None,
            funding_rate: None,
            open_interest: None,
            long_short_ratio: None,
            volatility_1h: None,
            volatility_24h: None,
            price_change_1h: None,
            price_change_24h: None,
            volume_change_1h: None,
            volume_change_24h: None,
            analyzed: false,
            usable_by_model: false,
            created_at: Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MarketDataIndicatorUpdate {
    pub id: Uuid,
    pub rsi_14: Option<Decimal>,
    pub macd_line: Option<Decimal>,
    pub macd_signal: Option<Decimal>,
    pub macd_histogram: Option<Decimal>,
    pub bb_upper: Option<Decimal>,
    pub bb_middle: Option<Decimal>,
    pub bb_lower: Option<Decimal>,
    pub atr_14: Option<Decimal>,
    pub depth_imbalance: Option<Decimal>,
    pub volatility_24h: Option<Decimal>,
    pub analyzed: bool,
    pub usable_by_model: bool,
}

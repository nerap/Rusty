use chrono::{DateTime, Utc};
use postgres_types::{FromSql, ToSql};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, PartialEq, FromSql, ToSql, Clone)]
#[postgres(name = "marketregime")]
pub enum MarketRegime {
    #[postgres(name = "none")]
    #[serde(rename = "none")]
    None,
    #[postgres(name = "trending_up")]
    #[serde(rename = "TRENDING_UP")]
    TrendingUp,
    #[postgres(name = "trending_down")]
    #[serde(rename = "TRENDING_DOWN")]
    TrendingDown,
    #[postgres(name = "ranging")]
    #[serde(rename = "RANGING")]
    Ranging,
    #[postgres(name = "high_volatility")]
    #[serde(rename = "HIGH_VOLATILITY")]
    HighVolatility,
    #[postgres(name = "low_volatility")]
    #[serde(rename = "LOW_VOLATILITY")]
    LowVolatility,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, FromSql, ToSql, Clone)]
#[postgres(name = "pricepattern")]
pub enum PricePattern {
    #[postgres(name = "double_top")]
    #[serde(rename = "DOUBLE_TOP")]
    DoubleTop,
    #[postgres(name = "double_bottom")]
    #[serde(rename = "DOUBLE_BOTTOM")]
    DoubleBottom,
    #[postgres(name = "head_and_shoulders")]
    #[serde(rename = "HEAD_AND_SHOULDERS")]
    HeadAndShoulders,
    #[postgres(name = "inverse_head_and_shoulders")]
    #[serde(rename = "INVERSE_HEAD_AND_SHOULDERS")]
    InverseHeadAndShoulders,
    #[postgres(name = "bullish_engulfing")]
    #[serde(rename = "BULLISH_ENGULFING")]
    BullishEngulfing,
    #[postgres(name = "bearish_engulfing")]
    #[serde(rename = "BEARISH_ENGULFING")]
    BearishEngulfing,
    #[postgres(name = "doji")]
    #[serde(rename = "DOJI")]
    Doji,
    #[postgres(name = "morning_star")]
    #[serde(rename = "MORNING_STAR")]
    MorningStar,
    #[postgres(name = "evening_star")]
    #[serde(rename = "EVENING_STAR")]
    EveningStar,
}

#[derive(Debug, Serialize, Deserialize, Validate, Clone)]
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

    // Market Regime
    pub market_regime: Option<MarketRegime>,

    // Trend Indicators
    pub adx: Option<Decimal>,
    pub dmi_plus: Option<Decimal>,
    pub dmi_minus: Option<Decimal>,
    pub trend_strength: Option<Decimal>,
    pub trend_direction: Option<i32>, // 1 for up, -1 for down, 0 for neutral

    // Support/Resistance
    pub support_levels: Option<Vec<Decimal>>,
    pub resistance_levels: Option<Vec<Decimal>>,
    pub nearest_support: Option<Decimal>,
    pub nearest_resistance: Option<Decimal>,

    // Price Patterns
    pub detected_patterns: Option<Vec<PricePattern>>,
    pub pattern_strength: Option<Decimal>,

    // Market microstructure
    pub depth_imbalance: Option<Decimal>,

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
            market_regime: None,
            adx: None,
            dmi_plus: None,
            dmi_minus: None,
            trend_strength: None,
            trend_direction: None,
            support_levels: None,
            resistance_levels: None,
            nearest_support: None,
            nearest_resistance: None,
            detected_patterns: None,
            pattern_strength: None,
            depth_imbalance: None,
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
    pub market_regime: Option<MarketRegime>,
    pub adx: Option<Decimal>,
    pub dmi_plus: Option<Decimal>,
    pub dmi_minus: Option<Decimal>,
    pub trend_strength: Option<Decimal>,
    pub trend_direction: Option<i32>, // 1 for up, -1 for down, 0 for neutral
    pub support_levels: Option<Vec<Decimal>>,
    pub resistance_levels: Option<Vec<Decimal>>,
    pub nearest_support: Option<Decimal>,
    pub nearest_resistance: Option<Decimal>,
    pub detected_patterns: Option<Vec<PricePattern>>,
    pub pattern_strength: Option<Decimal>,
    pub depth_imbalance: Option<Decimal>,
    pub volatility_1h: Option<Decimal>,
    pub volatility_24h: Option<Decimal>,
    pub price_change_1h: Option<Decimal>,
    pub price_change_24h: Option<Decimal>,
    pub volume_change_1h: Option<Decimal>,
    pub volume_change_24h: Option<Decimal>,
    pub analyzed: bool,
    pub usable_by_model: bool,
}

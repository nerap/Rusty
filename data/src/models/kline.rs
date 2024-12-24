use bytes::BytesMut;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tokio_postgres::types::{FromSql, IsNull, ToSql, Type};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub id: i64,
    pub symbol: String,
    pub contract_type: String,
    pub open_time: DateTime<Utc>,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub close_price: Decimal,
    pub volume: Decimal,
    pub base_asset_volume: Decimal,
    pub number_of_trades: i64,
    pub taker_buy_volume: Decimal,
    pub taker_buy_base_asset_volume: Decimal,

    // Price Action Indicators
    pub rsi_14: Option<f64>,
    pub rsi_4: Option<f64>,
    pub stoch_k_14: Option<f64>,
    pub stoch_d_14: Option<f64>,
    pub cci_20: Option<f64>,

    // Trend Indicators
    pub macd_12_26: Option<f64>,
    pub macd_signal_9: Option<f64>,
    pub macd_histogram: Option<f64>,
    pub ema_9: Option<f64>,
    pub ema_20: Option<f64>,
    pub ema_50: Option<f64>,
    pub ema_200: Option<f64>,

    // Volatility Indicators
    pub bollinger_upper: Option<f64>,
    pub bollinger_middle: Option<f64>,
    pub bollinger_lower: Option<f64>,
    pub atr_14: Option<f64>,
    pub keltner_upper: Option<f64>,
    pub keltner_middle: Option<f64>,
    pub keltner_lower: Option<f64>,

    // Volume Indicators
    pub obv: Option<Decimal>,
    pub mfi_14: Option<f64>,
    pub vwap: Option<Decimal>,
    pub cmf_20: Option<f64>,

    // Market Context
    pub funding_rate: Option<Decimal>,
    pub open_interest: Option<Decimal>,
    pub long_short_ratio: Option<f64>,
    pub cvd: Option<Decimal>,

    // Position Management
    pub current_position: Option<PositionType>,
    pub position_entry_price: Option<Decimal>,
    pub position_size: Option<Decimal>,
    pub position_pnl: Option<Decimal>,
    pub position_entry_time: Option<DateTime<Utc>>,

    // Prediction and Performance
    pub predicted_position: Option<PredictedPosition>,
    pub prediction_confidence: Option<f64>,
    pub actual_profit_loss: Option<Decimal>,
    pub trade_executed: bool,

    // Analyzed
    pub analyzed: Option<bool>,

    // Predicted
    pub predicted: Option<bool>,

    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineCreatePayload {
    pub symbol: String,
    pub contract_type: String,
    pub open_time: DateTime<Utc>,
    pub close_time: DateTime<Utc>,
    pub open_price: Decimal,
    pub high_price: Decimal,
    pub low_price: Decimal,
    pub close_price: Decimal,
    pub volume: Decimal,
    pub base_asset_volume: Decimal,
    pub number_of_trades: i64,
    pub taker_buy_volume: Decimal,
    pub taker_buy_base_asset_volume: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PositionType {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PredictedPosition {
    Long,
    Short,
    Hold,
}

impl<'a> FromSql<'a> for PositionType {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let s = String::from_utf8(raw.to_vec())?;
        match s.to_uppercase().as_str() {
            "LONG" => Ok(PositionType::Long),
            "SHORT" => Ok(PositionType::Short),
            _ => Err("Invalid position type".into()),
        }
    }

    fn accepts(ty: &Type) -> bool {
        matches!(ty, &Type::VARCHAR | &Type::TEXT)
    }
}

impl ToSql for PositionType {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let s = match self {
            PositionType::Long => "LONG",
            PositionType::Short => "SHORT",
        };
        out.extend_from_slice(s.as_bytes());
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        matches!(ty, &Type::VARCHAR | &Type::TEXT)
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}

impl<'a> FromSql<'a> for PredictedPosition {
    fn from_sql(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let s = String::from_utf8(raw.to_vec())?;
        match s.to_uppercase().as_str() {
            "LONG" => Ok(PredictedPosition::Long),
            "SHORT" => Ok(PredictedPosition::Short),
            "HOLD" => Ok(PredictedPosition::Hold),
            _ => Err("Invalid predicted position".into()),
        }
    }

    fn accepts(ty: &Type) -> bool {
        matches!(ty, &Type::VARCHAR | &Type::TEXT)
    }
}

impl ToSql for PredictedPosition {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let s = match self {
            PredictedPosition::Long => "LONG",
            PredictedPosition::Short => "SHORT",
            PredictedPosition::Hold => "HOLD",
        };
        out.extend_from_slice(s.as_bytes());
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        matches!(ty, &Type::VARCHAR | &Type::TEXT)
    }

    fn to_sql_checked(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        self.to_sql(ty, out)
    }
}

impl Kline {
    pub fn from_payload(payload: KlineCreatePayload) -> Self {
        Self {
            id: 0, // Will be set by database
            symbol: payload.symbol,
            contract_type: payload.contract_type,
            open_time: payload.open_time,
            open_price: payload.open_price,
            high_price: payload.high_price,
            low_price: payload.low_price,
            close_price: payload.close_price,
            volume: payload.volume,
            base_asset_volume: payload.base_asset_volume,
            number_of_trades: payload.number_of_trades,
            taker_buy_volume: payload.taker_buy_volume,
            taker_buy_base_asset_volume: payload.taker_buy_base_asset_volume,
            rsi_14: None,
            rsi_4: None,
            stoch_k_14: None,
            stoch_d_14: None,
            cci_20: None,
            macd_12_26: None,
            macd_signal_9: None,
            macd_histogram: None,
            ema_9: None,
            ema_20: None,
            ema_50: None,
            ema_200: None,
            bollinger_upper: None,
            bollinger_middle: None,
            bollinger_lower: None,
            atr_14: None,
            keltner_upper: None,
            keltner_middle: None,
            keltner_lower: None,
            obv: None,
            mfi_14: None,
            vwap: None,
            cmf_20: None,
            funding_rate: None,
            open_interest: None,
            long_short_ratio: None,
            cvd: None,
            current_position: None,
            position_entry_price: None,
            position_size: None,
            position_pnl: None,
            position_entry_time: None,
            predicted_position: None,
            prediction_confidence: None,
            actual_profit_loss: None,
            trade_executed: false,
            created_at: Utc::now(),
        }
    }
}

// Optional: Add validation trait
pub trait Validate {
    fn validate(&self) -> Result<(), String>;
}

impl Validate for KlineCreatePayload {
    fn validate(&self) -> Result<(), String> {
        // Price validations
        if self.open_price <= Decimal::ZERO {
            return Err("Open price must be positive".to_string());
        }
        if self.high_price <= Decimal::ZERO {
            return Err("High price must be positive".to_string());
        }
        if self.low_price <= Decimal::ZERO {
            return Err("Low price must be positive".to_string());
        }
        if self.close_price <= Decimal::ZERO {
            return Err("Close price must be positive".to_string());
        }

        // Volume validations
        if self.volume < Decimal::ZERO {
            return Err("Volume cannot be negative".to_string());
        }
        if self.base_asset_volume < Decimal::ZERO {
            return Err("Base asset volume cannot be negative".to_string());
        }

        // High/Low price relationship
        if self.high_price < self.low_price {
            return Err("High price cannot be less than low price".to_string());
        }

        // Symbol validation
        if self.symbol.trim().is_empty() {
            return Err("Symbol cannot be empty".to_string());
        }

        Ok(())
    }
}

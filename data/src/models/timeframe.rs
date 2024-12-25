use std::str::FromStr;

use chrono::{DateTime, Utc};
use postgres_types::{FromSql, ToSql};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::services::configuration_service::ConfigError;

#[derive(Debug, Serialize, Deserialize, PartialEq, FromSql, ToSql, Clone)]
#[postgres(name = "contracttype")]
pub enum ContractType {
    #[postgres(name = "perpetual")] // These should match the postgres enum values exactly
    #[serde(rename = "PERPETUAL")]
    Perpetual,

    #[postgres(name = "current_quarter")]
    #[serde(rename = "CURRENT_QUARTER")]
    CurrentQuarter,

    #[postgres(name = "next_quarter")]
    #[serde(rename = "NEXT_QUARTER")]
    NextQuarter,
}

impl ToString for ContractType {
    fn to_string(&self) -> String {
        match self {
            Self::Perpetual => "PERPETUAL".to_string(),
            Self::CurrentQuarter => "CURRENT_QUARTER".to_string(),
            Self::NextQuarter => "NEXT_QUARTER".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Interval {
    Minute1,
    Minute3,
    Minute5,
    Minute15,
    Minute30,
    Hour1,
    Hour2,
    Hour4,
    Hour6,
    Hour8,
    Hour12,
    Day1,
    Day3,
    Week1,
}

impl FromStr for Interval {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "1M" => Ok(Self::Minute1),
            "3M" => Ok(Self::Minute3),
            "5M" => Ok(Self::Minute5),
            "15M" => Ok(Self::Minute15),
            "30M" => Ok(Self::Minute30),
            "1H" => Ok(Self::Hour1),
            "2H" => Ok(Self::Hour2),
            "4H" => Ok(Self::Hour4),
            "6H" => Ok(Self::Hour6),
            "8H" => Ok(Self::Hour8),
            "12H" => Ok(Self::Hour12),
            "1D" => Ok(Self::Day1),
            "3D" => Ok(Self::Day3),
            "1W" => Ok(Self::Week1),
            _ => Err(ConfigError::InvalidInterval(s.to_string())),
        }
    }
}

impl ToString for Interval {
    fn to_string(&self) -> String {
        match self {
            Self::Minute1 => "1m",
            Self::Minute3 => "3m",
            Self::Minute5 => "5m",
            Self::Minute15 => "15m",
            Self::Minute30 => "30m",
            Self::Hour1 => "1h",
            Self::Hour2 => "2h",
            Self::Hour4 => "4h",
            Self::Hour6 => "6h",
            Self::Hour8 => "8h",
            Self::Hour12 => "12h",
            Self::Day1 => "1d",
            Self::Day3 => "3d",
            Self::Week1 => "1w",
        }
        .to_string()
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TimeFrame {
    pub id: Uuid,
    pub symbol: String,
    pub contract_type: ContractType,
    pub interval_minutes: i32,
    pub weight: Decimal,
    pub created_at: DateTime<Utc>,
}

impl TimeFrame {
    pub fn new(
        symbol: String,
        contract_type: ContractType,
        interval_minutes: i32,
        weight: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol,
            contract_type,
            interval_minutes,
            weight,
            created_at: Utc::now(),
        }
    }
}

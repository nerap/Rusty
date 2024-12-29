use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use postgres_types::{FromSql, ToSql};
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

impl fmt::Display for ContractType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Perpetual => write!(f, "PERPETUAL"),
            Self::CurrentQuarter => write!(f, "CURRENT_QUARTER"),
            Self::NextQuarter => write!(f, "NEXT_QUARTER"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
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
        match s.to_lowercase().as_str() {
            "1m" => Ok(Self::Minute1),
            "3m" => Ok(Self::Minute3),
            "5m" => Ok(Self::Minute5),
            "15m" => Ok(Self::Minute15),
            "30m" => Ok(Self::Minute30),
            "1h" => Ok(Self::Hour1),
            "2h" => Ok(Self::Hour2),
            "4h" => Ok(Self::Hour4),
            "6h" => Ok(Self::Hour6),
            "8h" => Ok(Self::Hour8),
            "12h" => Ok(Self::Hour12),
            "1d" => Ok(Self::Day1),
            "3d" => Ok(Self::Day3),
            "1w" => Ok(Self::Week1),
            _ => Err(ConfigError::InvalidInterval(s.to_string())),
        }
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
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
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TimeFrame {
    pub id: Uuid,
    pub symbol: String,
    pub contract_type: ContractType,
    pub interval_minutes: i32,
    pub created_at: DateTime<Utc>,
}

impl TimeFrame {
    pub fn new(symbol: String, contract_type: ContractType, interval_minutes: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            symbol,
            contract_type,
            interval_minutes,
            created_at: Utc::now(),
        }
    }
}

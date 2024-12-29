use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::timeframe::{ContractType, Interval};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid interval format: {0}")]
    InvalidInterval(String),
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub data: TradingConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TradingConfig {
    pub lookback_days: u32,
    pub pairs: Vec<PairConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PairConfig {
    pub symbol: String,
    pub contract_type: ContractType,
    pub timeframes: Vec<TimeframeConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeframeConfig {
    #[serde(with = "interval_string")]
    pub interval: Interval,
}

mod interval_string {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Interval, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }

    pub fn serialize<S>(interval: &Interval, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:?}", interval))
    }
}

pub struct ConfigService;

impl ConfigService {
    pub fn load_config(yaml: &str) -> Result<Config, ConfigError> {
        let config: Config = serde_yaml::from_str(yaml)?;
        Ok(config)
    }
}

use anyhow::{anyhow, Result};
use std::sync::Arc;

use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};

use crate::{
    lib::helper::Helper, models::market_data::MarketDataIndicatorUpdate,
    repositories::market_data_repository::MarketDataRepository,
};

use super::database_service::DatabaseService;

const DEFAULT_FECTH_LIMIT: i8 = 100;
const MANDATORY_RECORD_COUNT: usize = 250;

pub struct MarketDataAnalyzer {
    market_data_repository: Arc<MarketDataRepository>,
}

impl MarketDataAnalyzer {
    pub async fn new() -> Result<Self> {
        let market_data_repository = MarketDataRepository::new(DatabaseService::new().await?);

        Ok(MarketDataAnalyzer {
            market_data_repository: Arc::new(market_data_repository),
        })
    }

    pub async fn analyze_market_data(&self) -> Result<i32> {
        let mut analyzed_count = 0;
        loop {
            let unanalyzed_data = self
                .market_data_repository
                .find_unanalyzed_market_data(DEFAULT_FECTH_LIMIT)
                .await?;
            if unanalyzed_data.is_empty() {
                break;
            }

            for market_data in unanalyzed_data {
                let historical_data = self
                    .market_data_repository
                    .get_historical_data(
                        market_data.timeframe_id,
                        &market_data.symbol,
                        &market_data.contract_type,
                        market_data.open_time,
                        250,
                    )
                    .await?;

                let usable = historical_data.len() >= MANDATORY_RECORD_COUNT;

                if !usable {
                    self.market_data_repository
                        .update_indicators(MarketDataIndicatorUpdate {
                            id: market_data.id,
                            rsi_14: None,
                            macd_line: None,
                            macd_signal: None,
                            macd_histogram: None,
                            bb_upper: None,
                            bb_middle: None,
                            bb_lower: None,
                            atr_14: None,
                            depth_imbalance: None,
                            volatility_24h: None,
                            analyzed: true,
                            usable_by_model: false,
                        })
                        .await?;
                    continue;
                }

                let closes: Vec<f64> = historical_data
                    .iter()
                    .map(|d| d.close.to_f64().unwrap())
                    .collect();

                let rsi = Helper::calculate_rsi(&closes, 14);
                let (macd_line, signal, hist) = Helper::calculate_macd(&closes);
                let (upper, middle, lower) = Helper::calculate_bollinger_bands(&closes, 20, 2.0);
                let atr = Helper::calculate_atr(&historical_data, 14);
                let depth_imbalance = Helper::calculate_depth_imbalance(&historical_data);
                let volatility = Helper::calculate_volatility(&closes, 24);

                self.market_data_repository
                    .update_indicators(MarketDataIndicatorUpdate {
                        id: market_data.id,
                        rsi_14: Some(Decimal::from_f64(rsi).unwrap_or_default()),
                        macd_line: Some(Decimal::from_f64(macd_line).unwrap_or_default()),
                        macd_signal: Some(Decimal::from_f64(signal).unwrap_or_default()),
                        macd_histogram: Some(Decimal::from_f64(hist).unwrap_or_default()),
                        bb_upper: Some(Decimal::from_f64(upper).unwrap_or_default()),
                        bb_middle: Some(Decimal::from_f64(middle).unwrap_or_default()),
                        bb_lower: Some(Decimal::from_f64(lower).unwrap_or_default()),
                        atr_14: Some(Decimal::from_f64(atr).unwrap_or_default()),
                        depth_imbalance: Some(
                            Decimal::from_f64(depth_imbalance).unwrap_or_default(),
                        ),
                        volatility_24h: Some(Decimal::from_f64(volatility).unwrap_or_default()),
                        analyzed: true,
                        usable_by_model: true,
                    })
                    .await?;

                analyzed_count += 1;
            }
        }

        Ok(analyzed_count)
    }
}

use anyhow::Result;
use std::sync::Arc;

use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};

use crate::{
    models::market_data::{MarketDataIndicatorUpdate, PricePattern},
    repositories::market_data_repository::MarketDataRepository,
    utils::helper::Helper,
};

use super::database_service::DatabaseService;

const DEFAULT_FECTH_LIMIT: i8 = 100;
const MANDATORY_RECORD_COUNT: usize = 250;

pub struct MarketDataAnalyzer {
    market_data_repository: Arc<MarketDataRepository>,
}

impl MarketDataAnalyzer {
    pub async fn new() -> Result<Self> {
        let database = DatabaseService::new().await?;
        let market_data_repository = MarketDataRepository::new(database.client);

        Ok(MarketDataAnalyzer {
            market_data_repository: Arc::new(market_data_repository),
        })
    }

    pub async fn analyze_market_data(&self) -> Result<i32> {
        let mut analyzed_count = 0;

        // Constants for market regime and pattern detection
        const VOLATILITY_THRESHOLD: f64 = 0.02; // 2% daily volatility threshold
        const TREND_STRENGTH_THRESHOLD: f64 = 25.0; // ADX threshold
        const SR_WINDOW_SIZE: usize = 20; // Window for S/R detection
        const SR_THRESHOLD: f64 = 0.02; // 2% threshold for S/R clustering

        loop {
            let unanalyzed_data = self
                .market_data_repository
                .find_market_data_for_analysis(DEFAULT_FECTH_LIMIT, 100)
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
                            analyzed: true,
                            usable_by_model: false,
                        })
                        .await?;
                    continue;
                }

                // Calculate existing indicators
                let closes: Vec<f64> = historical_data
                    .iter()
                    .map(|d| d.close.to_f64().unwrap())
                    .collect();

                let rsi = Helper::calculate_rsi(&closes, 14);
                let (macd_line, signal, hist) = Helper::calculate_macd(&closes);
                let (upper, middle, lower) = Helper::calculate_bollinger_bands(&closes, 20, 2.0);
                let atr = Helper::calculate_atr(&historical_data, 14);
                let depth_imbalance = Helper::calculate_depth_imbalance(&historical_data);
                let volatility_1h = Helper::calculate_volatility(&closes, 1);
                let volatility_24h = Helper::calculate_volatility(&closes, 24);
                let price_change_1h = Helper::calculate_price_change(&historical_data, 1);
                let price_change_24h = Helper::calculate_price_change(&historical_data, 24);
                let volume_change_1h = Helper::calculate_volume_change(&historical_data, 1);
                let volume_change_24h = Helper::calculate_volume_change(&historical_data, 24);

                // Calculate new technical indicators
                let adx = Helper::calculate_adx(&historical_data, 14);
                let price_direction = Helper::calculate_price_direction(&historical_data, 20);

                // Detect market regime
                let market_regime = Helper::identify_market_regime(
                    &historical_data,
                    VOLATILITY_THRESHOLD,
                    TREND_STRENGTH_THRESHOLD,
                );

                // Find support and resistance levels
                let (support_levels, resistance_levels) = Helper::calculate_support_resistance(
                    &historical_data,
                    SR_WINDOW_SIZE,
                    SR_THRESHOLD,
                );

                // Convert levels to Decimal vectors
                let support_decimals = support_levels
                    .iter()
                    .map(|&x| Decimal::from_f64(x).unwrap())
                    .collect::<Vec<Decimal>>();

                let resistance_decimals = resistance_levels
                    .iter()
                    .map(|&x| Decimal::from_f64(x).unwrap())
                    .collect::<Vec<Decimal>>();

                // Find nearest support and resistance
                let current_price = historical_data[0].close.to_f64().unwrap();
                let nearest_support = support_levels
                    .iter()
                    .filter(|&&x| x < current_price)
                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|&x| Decimal::from_f64(x).unwrap());

                let nearest_resistance = resistance_levels
                    .iter()
                    .filter(|&&x| x > current_price)
                    .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                    .map(|&x| Decimal::from_f64(x).unwrap());

                let (dmi_plus, dmi_minus) = Helper::calculate_dmi(&historical_data, 14);

                const VOLUME_THRESHOLD: f64 = 1.5; // 150% of average volume
                let mut detected_patterns = Vec::new();
                let mut max_pattern_strength: f32 = 0.0;

                // Check each pattern type
                let patterns_to_check = [
                    PricePattern::DoubleTop,
                    PricePattern::DoubleBottom,
                    PricePattern::HeadAndShoulders,
                    PricePattern::InverseHeadAndShoulders,
                    PricePattern::BullishEngulfing,
                    PricePattern::BearishEngulfing,
                    PricePattern::Doji,
                    PricePattern::MorningStar,
                    PricePattern::EveningStar,
                ];

                for pattern in patterns_to_check.iter() {
                    if let Some(strength) = Helper::calculate_pattern_strength(
                        &historical_data,
                        pattern,
                        VOLUME_THRESHOLD,
                    ) {
                        if strength > 0.3 {
                            detected_patterns.push(pattern.clone());
                            max_pattern_strength = max_pattern_strength.max(strength as f32);
                        }
                    }
                }

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
                        market_regime,
                        adx: Some(Decimal::from_f64(adx).unwrap_or_default()),
                        dmi_plus: Some(Decimal::from_f64(dmi_plus).unwrap_or_default()),
                        dmi_minus: Some(Decimal::from_f64(dmi_minus).unwrap_or_default()),
                        trend_strength: Some(Decimal::from_f64(adx).unwrap_or_default()),
                        trend_direction: Some(price_direction as i32),
                        support_levels: Some(support_decimals),
                        resistance_levels: Some(resistance_decimals),
                        nearest_support,
                        nearest_resistance,
                        detected_patterns: Some(detected_patterns.clone()),
                        pattern_strength: if !detected_patterns.is_empty() {
                            Some(Decimal::from_f64(max_pattern_strength.into()).unwrap_or_default())
                        } else {
                            None
                        },
                        depth_imbalance: Some(
                            Decimal::from_f64(depth_imbalance).unwrap_or_default(),
                        ),
                        volatility_1h: Some(Decimal::from_f64(volatility_1h).unwrap_or_default()),
                        volatility_24h: Some(Decimal::from_f64(volatility_24h).unwrap_or_default()),
                        price_change_1h: Some(price_change_1h),
                        price_change_24h: Some(price_change_24h),
                        volume_change_1h: Some(volume_change_1h),
                        volume_change_24h: Some(volume_change_24h),
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

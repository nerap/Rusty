use chrono::Duration;
use rust_decimal::{prelude::ToPrimitive, Decimal};

use crate::models::market_data::MarketData;

pub struct Helper {}

impl Helper {
    pub fn minutes_to_interval(minutes: i32) -> String {
        match minutes {
            m if m < 60 => format!("{}m", m),
            m if m % (24 * 60) == 0 => format!("{}d", m / (24 * 60)),
            m if m % 60 == 0 => format!("{}h", m / 60),
            m if m % (7 * 24 * 60) == 0 => format!("{}w", m / (7 * 24 * 60)),
            _ => format!("{}m", minutes),
        }
    }

    pub fn interval_to_minutes(interval: &str) -> Option<i32> {
        let len = interval.len();
        if len < 2 {
            return None;
        }

        let (value_str, unit) = interval.split_at(len - 1);
        let value: i32 = value_str.parse().ok()?;

        match unit {
            "m" => Some(value),
            "h" => Some(value * 60),
            "d" => Some(value * 24 * 60),
            "w" => Some(value * 7 * 24 * 60),
            _ => None,
        }
    }

    // Indicator calculations
    pub fn calculate_rsi(closes: &[f64], period: usize) -> f64 {
        let mut gains = vec![0.0];
        let mut losses = vec![0.0];

        for i in 1..closes.len() {
            let diff = closes[i] - closes[i - 1];
            gains.push(diff.max(0.0));
            losses.push((-diff).max(0.0));
        }

        let avg_gain = gains.iter().take(period).sum::<f64>() / period as f64;
        let avg_loss = losses.iter().take(period).sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
    }

    pub fn calculate_macd(closes: &[f64]) -> (f64, f64, f64) {
        let fast_period = 12;
        let slow_period = 26;
        let signal_period = 9;

        // Calculate EMAs for entire series
        let mut fast_emas = Vec::with_capacity(closes.len());
        let mut slow_emas = Vec::with_capacity(closes.len());
        let mut macd_lines = Vec::with_capacity(closes.len());

        for i in 0..closes.len() {
            let slice = &closes[0..=i];
            fast_emas.push(Helper::exponential_ma(slice, fast_period));
            slow_emas.push(Helper::exponential_ma(slice, slow_period));
            macd_lines.push(fast_emas[i] - slow_emas[i]);
        }

        // Calculate signal line from MACD values
        let signal = Helper::exponential_ma(&macd_lines, signal_period);
        let macd_line = *macd_lines.last().unwrap();
        let histogram = macd_line - signal;

        (macd_line, signal, histogram)
    }

    pub fn calculate_bollinger_bands(
        closes: &[f64],
        period: usize,
        std_dev: f64,
    ) -> (f64, f64, f64) {
        let sma = Helper::simple_ma(closes, period);
        let std = Helper::standard_deviation(closes, period);

        let upper = sma + std_dev * std;
        let lower = sma - std_dev * std;

        (upper, sma, lower)
    }

    pub fn calculate_atr(data: &[MarketData], period: usize) -> f64 {
        let mut tr = Vec::with_capacity(data.len());

        for i in 1..data.len() {
            let high = data[i].high.to_f64().unwrap();
            let low = data[i].low.to_f64().unwrap();
            let prev_close = data[i - 1].close.to_f64().unwrap();

            let tr_1 = high - low;
            let tr_2 = (high - prev_close).abs();
            let tr_3 = (low - prev_close).abs();

            tr.push(tr_1.max(tr_2).max(tr_3));
        }

        Helper::exponential_ma(&tr, period)
    }

    pub fn calculate_volatility(closes: &[f64], hours: i32) -> f64 {
        let returns: Vec<f64> = closes.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();

        let period = if returns.len() >= (hours * 60) as usize {
            (hours * 60) as usize // Convert hours to minutes for candle count
        } else {
            returns.len()
        };

        Helper::standard_deviation(&returns, period) * (252_f64 * 24.0 / hours as f64).sqrt()
        // Annualized based on hours
    }

    pub fn calculate_price_change(data: &[MarketData], hours: i64) -> Decimal {
        if data.len() < 2 {
            return Decimal::ZERO;
        }

        let target_time = data[0].open_time - Duration::hours(hours);
        let old_price = match data.iter().find(|d| d.open_time <= target_time) {
            Some(d) => d.close,
            None => return Decimal::ZERO,
        };

        ((data[0].close - old_price) / old_price) * Decimal::ONE_HUNDRED
    }

    pub fn calculate_volume_change(data: &[MarketData], hours: i64) -> Decimal {
        if data.len() < 2 {
            return Decimal::ZERO;
        }

        let target_time = data[0].open_time - Duration::hours(hours);
        let old_volume = match data.iter().find(|d| d.open_time <= target_time) {
            Some(d) => d.volume,
            None => return Decimal::ZERO,
        };

        ((data[0].volume - old_volume) / old_volume) * Decimal::ONE_HUNDRED
    }

    pub fn calculate_depth_imbalance(data: &[MarketData]) -> f64 {
        let volumes: Vec<f64> = data.iter().map(|d| d.volume.to_f64().unwrap()).collect();

        let prices: Vec<f64> = data.iter().map(|d| d.close.to_f64().unwrap()).collect();

        let vol_ma = Helper::simple_ma(&volumes, 24);
        let price_std = Helper::standard_deviation(&prices, 24);

        vol_ma * price_std
    }
    // Helper functions
    pub fn exponential_ma(values: &[f64], period: usize) -> f64 {
        let alpha = 2.0 / (period + 1) as f64;
        let mut ema = values[0];

        for &value in &values[1..] {
            ema = value * alpha + ema * (1.0 - alpha);
        }

        ema
    }

    pub fn simple_ma(values: &[f64], period: usize) -> f64 {
        values.iter().take(period).sum::<f64>() / period as f64
    }

    pub fn standard_deviation(values: &[f64], period: usize) -> f64 {
        let mean = Helper::simple_ma(values, period);
        let variance = values
            .iter()
            .take(period)
            .map(|x| (x - mean).powi(2))
            .sum::<f64>()
            / period as f64;
        variance.sqrt()
    }
}

use chrono::Duration;
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive},
    Decimal,
};
use thiserror::Error;

use crate::models::market_data::{MarketData, MarketRegime, PricePattern};

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

        if closes.len() < 2 || period == 0 {
            return 50.0; // Default neutral value
        }

        for i in 1..closes.len() {
            let diff = closes[i] - closes[i - 1];
            gains.push(diff.max(0.0));
            losses.push((-diff).max(0.0));
        }

        let avg_gain = gains.iter().take(period).sum::<f64>() / period as f64;
        let avg_loss = losses.iter().take(period).sum::<f64>() / period as f64;

        if avg_loss == 0.0 {
            if avg_gain == 0.0 {
                return 50.0;
            }
            return 100.0;
        }
        100.0 - (100.0 / (1.0 + avg_gain / avg_loss))
    }

    pub fn calculate_macd(closes: &[f64]) -> (f64, f64, f64) {
        let fast_period = 12;
        let slow_period = 26;
        let signal_period = 9;

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
            (hours * 60) as usize
        } else {
            returns.len()
        };

        Helper::standard_deviation(&returns, period) * (252_f64 * 24.0 / hours as f64).sqrt()
    }

    pub fn calculate_price_change(data: &[MarketData], hours: i64) -> Decimal {
        if data.len() < 2 || hours <= 0 {
            return Decimal::ZERO;
        }
        let target_time = data[0].open_time - Duration::hours(hours);
        let old_price = match data.iter().find(|d| d.open_time <= target_time) {
            Some(d) => d.close,
            None => return Decimal::ZERO,
        };

        ((data[0].close - old_price) / old_price) * Decimal::ONE_HUNDRED
    }

    pub fn calculate_price_range(data: &[MarketData]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }

        let high = data
            .iter()
            .map(|d| d.high.to_f64().unwrap())
            .fold(f64::MIN, f64::max);

        let low = data
            .iter()
            .map(|d| d.low.to_f64().unwrap())
            .fold(f64::MAX, f64::min);

        let avg_price =
            data.iter().map(|d| d.close.to_f64().unwrap()).sum::<f64>() / data.len() as f64;

        let basic_range = (high - low) / avg_price;

        let volatility = Self::calculate_price_volatility(data);
        let volatility_factor = if volatility > 0.0 {
            1.0 + (volatility / 0.1)
        } else {
            1.0
        };

        let time_factor = Self::calculate_time_factor(data);

        let adjusted_range = basic_range * volatility_factor * time_factor;

        adjusted_range.min(1.0).max(0.0)
    }

    fn calculate_price_volatility(data: &[MarketData]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }

        let returns: Vec<f64> = data
            .windows(2)
            .map(|window| {
                let current = window[0].close.to_f64().unwrap();
                let previous = window[1].close.to_f64().unwrap();
                (current - previous) / previous
            })
            .collect();

        let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns
            .iter()
            .map(|r| (r - mean_return).powi(2))
            .sum::<f64>()
            / returns.len() as f64;

        variance.sqrt()
    }

    fn calculate_time_factor(data: &[MarketData]) -> f64 {
        let pattern_duration = data.len();

        let ideal_duration = 20;

        if pattern_duration < ideal_duration {
            0.5 + (0.5 * pattern_duration as f64 / ideal_duration as f64)
        } else if pattern_duration > ideal_duration * 2 {
            0.75
        } else {
            1.0
        }
    }

    pub fn calculate_volume_change(data: &[MarketData], hours: i64) -> Decimal {
        if data.len() < 2 || hours <= 0 {
            return Decimal::ZERO;
        }

        let target_time = data[0].open_time - Duration::hours(hours);
        let old_volume = match data.iter().find(|d| d.open_time <= target_time) {
            Some(d) => d.volume,
            None => return Decimal::ZERO,
        };

        if old_volume == Decimal::ZERO {
            return Decimal::ZERO;
        }
        ((data[0].volume - old_volume) / old_volume) * Decimal::ONE_HUNDRED
    }

    pub fn calculate_depth_imbalance(data: &[MarketData]) -> f64 {
        let volumes: Vec<f64> = data.iter().map(|d| d.volume.to_f64().unwrap()).collect();

        let prices: Vec<f64> = data.iter().map(|d| d.close.to_f64().unwrap()).collect();

        let vol_ma = Helper::simple_ma(&volumes, 24);
        let price_std = Helper::standard_deviation(&prices, 24);

        vol_ma * price_std
    }
    pub fn exponential_ma(values: &[f64], period: usize) -> f64 {
        let alpha = 2.0 / (period + 1) as f64;
        let mut ema = values[0];

        for &value in &values[1..] {
            ema = value * alpha + ema * (1.0 - alpha);
        }

        ema
    }

    pub fn simple_ma(values: &[f64], period: usize) -> f64 {
        if values.is_empty() || period == 0 {
            return 0.0;
        }
        let period = period.min(values.len());
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

    pub fn identify_market_regime(
        data: &[MarketData],
        volatility_threshold: f64,
        trend_strength_threshold: f64,
    ) -> Option<MarketRegime> {
        if data.len() < 20 {
            return None;
        }

        let adx = Self::calculate_adx(data, 14);
        let current_volatility = data[0].volatility_24h.unwrap_or_default().to_f64().unwrap();
        let price_direction = Self::calculate_price_direction(data, 20);

        match (adx, current_volatility, price_direction) {
            (_adx, vol, _dir) if vol > volatility_threshold => Some(MarketRegime::HighVolatility),
            (_adx, vol, _dir) if vol < volatility_threshold * 0.5 => {
                Some(MarketRegime::LowVolatility)
            }
            (adx, _, dir) if adx > trend_strength_threshold && dir > 0.0 => {
                Some(MarketRegime::TrendingUp)
            }
            (adx, _, dir) if adx > trend_strength_threshold && dir < 0.0 => {
                Some(MarketRegime::TrendingDown)
            }
            _ => Some(MarketRegime::Ranging),
        }
    }

    pub fn calculate_adx(data: &[MarketData], period: usize) -> f64 {
        if data.len() < period * 2 {
            return 0.0;
        }

        let mut tr_values = Vec::with_capacity(data.len());
        let mut plus_dm = Vec::with_capacity(data.len());
        let mut minus_dm = Vec::with_capacity(data.len());

        for i in 1..data.len() {
            let high = data[i].high.to_f64().unwrap();
            let low = data[i].low.to_f64().unwrap();
            let prev_high = data[i - 1].high.to_f64().unwrap();
            let prev_low = data[i - 1].low.to_f64().unwrap();
            let prev_close = data[i - 1].close.to_f64().unwrap();

            let tr = (high - low)
                .max((high - prev_close).abs())
                .max((low - prev_close).abs());
            tr_values.push(tr);

            let up_move = high - prev_high;
            let down_move = prev_low - low;

            if up_move > down_move && up_move > 0.0 {
                plus_dm.push(up_move);
                minus_dm.push(0.0);
            } else if down_move > up_move && down_move > 0.0 {
                plus_dm.push(0.0);
                minus_dm.push(down_move);
            } else {
                plus_dm.push(0.0);
                minus_dm.push(0.0);
            }
        }

        let mut smoothed_tr = tr_values[0..period].iter().sum::<f64>();
        let mut smoothed_plus_dm = plus_dm[0..period].iter().sum::<f64>();
        let mut smoothed_minus_dm = minus_dm[0..period].iter().sum::<f64>();

        let mut adx_values = Vec::with_capacity(data.len() - period);

        for i in period..data.len() {
            smoothed_tr = smoothed_tr - (smoothed_tr / period as f64) + tr_values[i];
            smoothed_plus_dm = smoothed_plus_dm - (smoothed_plus_dm / period as f64) + plus_dm[i];
            smoothed_minus_dm =
                smoothed_minus_dm - (smoothed_minus_dm / period as f64) + minus_dm[i];

            let plus_di = 100.0 * (smoothed_plus_dm / smoothed_tr);
            let minus_di = 100.0 * (smoothed_minus_dm / smoothed_tr);

            let dx = 100.0 * (plus_di - minus_di).abs() / (plus_di + minus_di);
            adx_values.push(dx);
        }

        Self::exponential_ma(&adx_values, period)
    }

    pub fn calculate_support_resistance(
        data: &[MarketData],
        window_size: usize,
        threshold: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let mut support_levels = Vec::new();
        let mut resistance_levels = Vec::new();

        for i in window_size..data.len() - window_size {
            let current_price = data[i].close.to_f64().unwrap();

            let is_support = (0..window_size).all(|j| {
                data[i - j].low.to_f64().unwrap() >= data[i].low.to_f64().unwrap()
                    && data[i + j].low.to_f64().unwrap() >= data[i].low.to_f64().unwrap()
            });

            let is_resistance = (0..window_size).all(|j| {
                data[i - j].high.to_f64().unwrap() <= data[i].high.to_f64().unwrap()
                    && data[i + j].high.to_f64().unwrap() <= data[i].high.to_f64().unwrap()
            });

            if is_support {
                support_levels.push(current_price);
            }
            if is_resistance {
                resistance_levels.push(current_price);
            }
        }

        let (support_levels, resistance_levels) =
            Self::cluster_levels(support_levels, resistance_levels, threshold);

        (support_levels, resistance_levels)
    }

    pub fn calculate_price_direction(data: &[MarketData], period: usize) -> f64 {
        if data.len() < period {
            return 0.0;
        }

        let closes: Vec<f64> = data
            .iter()
            .take(period)
            .map(|d| d.close.to_f64().unwrap())
            .collect();

        let short_period = period / 4;
        let short_ma = Self::exponential_ma(&closes, short_period);
        let long_ma = Self::exponential_ma(&closes, period);

        if (short_ma - long_ma).abs() / long_ma < 0.001 {
            // If MAs are very close, consider it neutral
            0.0
        } else if short_ma > long_ma {
            1.0 // Uptrend
        } else {
            -1.0 // Downtrend
        }
    }

    pub fn cluster_levels(
        mut supports: Vec<f64>,
        mut resistances: Vec<f64>,
        threshold: f64,
    ) -> (Vec<f64>, Vec<f64>) {
        let mut clustered_supports = Vec::new();
        let mut clustered_resistances = Vec::new();

        while !supports.is_empty() {
            let base = supports[0];
            let mut cluster = Vec::new();

            let mut remaining = Vec::new();
            for &price in supports.iter() {
                if (price - base).abs() / base < threshold {
                    cluster.push(price);
                } else {
                    remaining.push(price);
                }
            }

            if !cluster.is_empty() {
                clustered_supports.push(cluster.iter().sum::<f64>() / cluster.len() as f64);
            }

            supports = remaining;
        }

        while !resistances.is_empty() {
            let base = resistances[0];
            let mut cluster = Vec::new();

            let mut remaining = Vec::new();
            for &price in resistances.iter() {
                if (price - base).abs() / base < threshold {
                    cluster.push(price);
                } else {
                    remaining.push(price);
                }
            }

            if !cluster.is_empty() {
                clustered_resistances.push(cluster.iter().sum::<f64>() / cluster.len() as f64);
            }

            resistances = remaining;
        }

        clustered_supports.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        clustered_resistances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        (clustered_supports, clustered_resistances)
    }

    pub fn is_bullish_engulfing(data: &[MarketData]) -> bool {
        if data.len() < 2 {
            return false;
        }

        let current = &data[0];
        let previous = &data[1];

        let prev_bearish = previous.close < previous.open;
        let curr_bullish = current.close > current.open;
        let engulfs = current.open < previous.close && current.close > previous.open;

        prev_bearish && curr_bullish && engulfs
    }

    pub fn is_bearish_engulfing(data: &[MarketData]) -> bool {
        if data.len() < 2 {
            return false;
        }

        let current = &data[0];
        let previous = &data[1];

        let prev_bullish = previous.close > previous.open;
        let curr_bearish = current.close < current.open;
        let engulfs = current.open > previous.close && current.close < previous.open;

        prev_bullish && curr_bearish && engulfs
    }
    pub fn is_doji(data: &[MarketData]) -> bool {
        if data.is_empty() {
            return false;
        }

        let candle = &data[0];
        let body_size = (candle.close - candle.open).abs();
        let total_size = candle.high - candle.low;

        // Body is very small compared to total size
        body_size / total_size < Decimal::from_f32(0.1).unwrap()
    }

    pub fn is_morning_star(data: &[MarketData]) -> bool {
        if data.len() < 3 {
            return false;
        }

        let first = &data[2]; // First day
        let second = &data[1]; // Second day (doji)
        let third = &data[0]; // Third day

        let first_bearish = first.close < first.open;
        let third_bullish = third.close > third.open;
        let is_second_doji = Self::is_doji(&[second.clone()]);

        // Gap down between first and second day
        let gap_down = second.high < first.close;
        // Gap up between second and third day
        let gap_up = third.open > second.low;

        first_bearish && is_second_doji && third_bullish && gap_down && gap_up
    }

    pub fn is_evening_star(data: &[MarketData]) -> bool {
        if data.len() < 3 {
            return false;
        }

        let first = &data[2]; // First day
        let second = &data[1]; // Second day (doji)
        let third = &data[0]; // Third day

        let first_bullish = first.close > first.open;
        let third_bearish = third.close < third.open;
        let is_second_doji = Self::is_doji(&[second.clone()]);

        // Gap up between first and second day
        let gap_up = second.low > first.close;
        // Gap down between second and third day
        let gap_down = third.open < second.high;

        first_bullish && is_second_doji && third_bearish && gap_up && gap_down
    }

    pub fn is_double_top(data: &[MarketData]) -> bool {
        if data.len() < 20 {
            return false;
        }

        let price_similarity_threshold = Decimal::from_f64(0.02).unwrap();
        let min_peak_distance = 5;
        let min_trough_depth = Decimal::from_f64(0.03).unwrap();

        let mut peaks: Vec<(usize, Decimal)> = Vec::new();
        for i in 2..data.len() - 2 {
            let current_high = data[i].high;

            if current_high > data[i - 1].high
                && current_high > data[i - 2].high
                && current_high > data[i + 1].high
                && current_high > data[i + 2].high
            {
                peaks.push((i, current_high));
            }
        }

        if peaks.len() < 2 {
            return false;
        }

        for i in 0..peaks.len() - 1 {
            for j in i + 1..peaks.len() {
                let (idx1, peak1) = peaks[i];
                let (idx2, peak2) = peaks[j];

                if idx2 - idx1 < min_peak_distance {
                    continue;
                }

                let price_diff = ((peak1 - peak2).abs() / peak1).abs();
                if price_diff > price_similarity_threshold {
                    continue;
                }

                let mut min_trough = Decimal::MAX;
                for k in idx1 + 1..idx2 {
                    min_trough = min_trough.min(data[k].low);
                }

                let avg_peak_height = (peak1 + peak2) / Decimal::from(2);
                let trough_depth = (avg_peak_height - min_trough) / avg_peak_height;

                if trough_depth >= min_trough_depth {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_double_bottom(data: &[MarketData]) -> bool {
        if data.len() < 20 {
            return false;
        }

        let price_similarity_threshold = Decimal::from_f64(0.02).unwrap();
        let min_trough_distance = 5;
        let min_peak_height = Decimal::from_f64(0.03).unwrap();

        let mut troughs: Vec<(usize, Decimal)> = Vec::new();
        for i in 2..data.len() - 2 {
            let current_low = data[i].low;

            if current_low < data[i - 1].low
                && current_low < data[i - 2].low
                && current_low < data[i + 1].low
                && current_low < data[i + 2].low
            {
                troughs.push((i, current_low));
            }
        }

        if troughs.len() < 2 {
            return false;
        }

        for i in 0..troughs.len() - 1 {
            for j in i + 1..troughs.len() {
                let (idx1, trough1) = troughs[i];
                let (idx2, trough2) = troughs[j];

                if idx2 - idx1 < min_trough_distance {
                    continue;
                }

                let price_diff = ((trough1 - trough2).abs() / trough1).abs();
                if price_diff > price_similarity_threshold {
                    continue;
                }

                let mut max_peak = Decimal::MIN;
                for k in idx1 + 1..idx2 {
                    max_peak = max_peak.max(data[k].high);
                }

                let avg_trough_depth = (trough1 + trough2) / Decimal::from(2);
                let peak_height = (max_peak - avg_trough_depth) / avg_trough_depth;

                if peak_height >= min_peak_height {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_head_and_shoulders(data: &[MarketData]) -> bool {
        if data.len() < 30 {
            return false;
        }

        let shoulder_similarity_threshold = Decimal::from_f64(0.03).unwrap();
        let min_peak_distance = 5;
        let head_height_min = Decimal::from_f64(0.02).unwrap();

        let mut peaks: Vec<(usize, Decimal)> = Vec::new();
        for i in 2..data.len() - 2 {
            let current_high = data[i].high;

            if current_high > data[i - 1].high
                && current_high > data[i - 2].high
                && current_high > data[i + 1].high
                && current_high > data[i + 2].high
            {
                peaks.push((i, current_high));
            }
        }

        if peaks.len() < 3 {
            return false;
        }

        for i in 0..peaks.len() - 2 {
            for j in i + 1..peaks.len() - 1 {
                for k in j + 1..peaks.len() {
                    let (left_idx, left_shoulder) = peaks[i];
                    let (head_idx, head) = peaks[j];
                    let (right_idx, right_shoulder) = peaks[k];

                    if head_idx - left_idx < min_peak_distance
                        || right_idx - head_idx < min_peak_distance
                    {
                        continue;
                    }

                    let shoulder_diff =
                        ((left_shoulder - right_shoulder).abs() / left_shoulder).abs();
                    if shoulder_diff > shoulder_similarity_threshold {
                        continue;
                    }

                    let avg_shoulder_height = (left_shoulder + right_shoulder) / Decimal::from(2);
                    let head_height = (head - avg_shoulder_height) / avg_shoulder_height;

                    if head_height >= head_height_min {
                        let mut left_trough = Decimal::MAX;
                        let mut right_trough = Decimal::MAX;

                        for idx in left_idx + 1..head_idx {
                            left_trough = left_trough.min(data[idx].low);
                        }

                        for idx in head_idx + 1..right_idx {
                            right_trough = right_trough.min(data[idx].low);
                        }

                        let trough_diff = ((left_trough - right_trough).abs() / left_trough).abs();
                        if trough_diff <= shoulder_similarity_threshold {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    pub fn is_inverse_head_and_shoulders(data: &[MarketData]) -> bool {
        if data.len() < 30 {
            return false;
        }

        let shoulder_similarity_threshold = Decimal::from_f64(0.03).unwrap();
        let min_trough_distance = 5;
        let head_depth_min = Decimal::from_f64(0.02).unwrap();

        let mut troughs: Vec<(usize, Decimal)> = Vec::new();
        for i in 2..data.len() - 2 {
            let current_low = data[i].low;

            if current_low < data[i - 1].low
                && current_low < data[i - 2].low
                && current_low < data[i + 1].low
                && current_low < data[i + 2].low
            {
                troughs.push((i, current_low));
            }
        }

        if troughs.len() < 3 {
            return false;
        }

        for i in 0..troughs.len() - 2 {
            for j in i + 1..troughs.len() - 1 {
                for k in j + 1..troughs.len() {
                    let (left_idx, left_shoulder) = troughs[i];
                    let (head_idx, head) = troughs[j];
                    let (right_idx, right_shoulder) = troughs[k];

                    if head_idx - left_idx < min_trough_distance
                        || right_idx - head_idx < min_trough_distance
                    {
                        continue;
                    }

                    let shoulder_diff =
                        ((left_shoulder - right_shoulder).abs() / left_shoulder).abs();
                    if shoulder_diff > shoulder_similarity_threshold {
                        continue;
                    }

                    let avg_shoulder_depth = (left_shoulder + right_shoulder) / Decimal::from(2);
                    let head_depth = (avg_shoulder_depth - head) / avg_shoulder_depth;

                    if head_depth >= head_depth_min {
                        let mut left_peak = Decimal::MIN;
                        let mut right_peak = Decimal::MIN;

                        for idx in left_idx + 1..head_idx {
                            left_peak = left_peak.max(data[idx].high);
                        }

                        for idx in head_idx + 1..right_idx {
                            right_peak = right_peak.max(data[idx].high);
                        }

                        let peak_diff = ((left_peak - right_peak).abs() / left_peak).abs();
                        if peak_diff <= shoulder_similarity_threshold {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }
    pub fn calculate_dmi(data: &[MarketData], period: usize) -> (f64, f64) {
        if data.len() < period * 2 {
            return (0.0, 0.0);
        }

        let mut tr_values = Vec::with_capacity(data.len());
        let mut plus_dm = Vec::with_capacity(data.len());
        let mut minus_dm = Vec::with_capacity(data.len());

        // Calculate TR and DM values
        for i in 1..data.len() {
            let high = data[i].high.to_f64().unwrap();
            let low = data[i].low.to_f64().unwrap();
            let prev_high = data[i - 1].high.to_f64().unwrap();
            let prev_low = data[i - 1].low.to_f64().unwrap();
            let prev_close = data[i - 1].close.to_f64().unwrap();

            // True Range
            let tr = (high - low)
                .max((high - prev_close).abs())
                .max((low - prev_close).abs());
            tr_values.push(tr);

            // Directional Movement
            let up_move = high - prev_high;
            let down_move = prev_low - low;

            if up_move > down_move && up_move > 0.0 {
                plus_dm.push(up_move);
                minus_dm.push(0.0);
            } else if down_move > up_move && down_move > 0.0 {
                plus_dm.push(0.0);
                minus_dm.push(down_move);
            } else {
                plus_dm.push(0.0);
                minus_dm.push(0.0);
            }
        }

        // Calculate smoothed values
        let smoothed_tr = Self::exponential_ma(&tr_values, period);
        let smoothed_plus_dm = Self::exponential_ma(&plus_dm, period);
        let smoothed_minus_dm = Self::exponential_ma(&minus_dm, period);

        // Calculate DMI values
        let plus_di = if smoothed_tr != 0.0 {
            100.0 * smoothed_plus_dm / smoothed_tr
        } else {
            0.0
        };
        let minus_di = if smoothed_tr != 0.0 {
            100.0 * smoothed_minus_dm / smoothed_tr
        } else {
            0.0
        };

        (plus_di, minus_di)
    }

    pub fn calculate_std_dev(values: &[f64]) -> f64 {
        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n;
        variance.sqrt()
    }

    pub fn calculate_pattern_strength(
        data: &[MarketData],
        pattern_type: &PricePattern,
        volume_threshold: f64,
    ) -> Option<f64> {
        let base_strength = match pattern_type {
            PricePattern::DoubleTop => {
                if Self::is_double_top(data) {
                    Some(Self::evaluate_pattern_strength(data, true))
                } else {
                    None
                }
            }
            PricePattern::DoubleBottom => {
                if Self::is_double_bottom(data) {
                    Some(Self::evaluate_pattern_strength(data, false))
                } else {
                    None
                }
            }
            PricePattern::HeadAndShoulders => {
                if Self::is_head_and_shoulders(data) {
                    Some(Self::evaluate_pattern_strength(data, true))
                } else {
                    None
                }
            }
            PricePattern::InverseHeadAndShoulders => {
                if Self::is_inverse_head_and_shoulders(data) {
                    Some(Self::evaluate_pattern_strength(data, false))
                } else {
                    None
                }
            }
            PricePattern::BullishEngulfing => {
                if Self::is_bullish_engulfing(data) {
                    Some(Self::evaluate_pattern_strength(data, true))
                } else {
                    None
                }
            }
            PricePattern::BearishEngulfing => {
                if Self::is_bearish_engulfing(data) {
                    Some(Self::evaluate_pattern_strength(data, true))
                } else {
                    None
                }
            }
            PricePattern::Doji => {
                if Self::is_doji(data) {
                    Some(Self::evaluate_pattern_strength(data, false))
                } else {
                    None
                }
            }
            PricePattern::MorningStar => {
                if Self::is_morning_star(data) {
                    Some(Self::evaluate_pattern_strength(data, true))
                } else {
                    None
                }
            }
            PricePattern::EveningStar => {
                if Self::is_evening_star(data) {
                    Some(Self::evaluate_pattern_strength(data, true))
                } else {
                    None
                }
            }
            PricePattern::None => None
        };

        base_strength.map(|strength| {
            let volume_confirmation = Self::check_volume_confirmation(data, volume_threshold);
            let trend_confirmation = Self::check_trend_confirmation(data);

            // Combine all factors for final strength
            strength * volume_confirmation * trend_confirmation
        })
    }

    pub fn check_volume_confirmation(data: &[MarketData], threshold: f64) -> f64 {
        let avg_volume =
            data.iter().map(|d| d.volume.to_f64().unwrap()).sum::<f64>() / data.len() as f64;

        let recent_volume = data[0].volume.to_f64().unwrap();
        let volume_ratio = recent_volume / avg_volume;

        if volume_ratio >= threshold {
            1.0
        } else {
            volume_ratio / threshold
        }
    }

    pub fn check_trend_confirmation(data: &[MarketData]) -> f64 {
        let adx = Self::calculate_adx(data, 14);
        let normalized_adx = adx / 100.0; // ADX ranges from 0 to 100

        // Strong trend > 25 ADX gets full score
        if normalized_adx > 0.25 {
            1.0
        } else {
            normalized_adx / 0.25
        }
    }

    pub fn normalize_range(value: f64, min: f64, max: f64) -> f64 {
        ((value - min) / (max - min)).max(0.0).min(1.0)
    }

    pub fn evaluate_pattern_strength(data: &[MarketData], is_reversal: bool) -> f64 {
        let mut strength = 1.0;

        // 1. Price Movement Magnitude
        let price_range = Self::calculate_price_range(data);
        let range_score = Self::normalize_range(price_range, 0.01, 0.1);
        strength *= range_score * 0.25; // 25% weight

        // 2. Volume Analysis
        let volume_score = Self::evaluate_volume_pattern(data);
        strength *= volume_score * 0.20; // 20% weight

        // 3. Pattern Formation Quality
        let quality_score = Self::evaluate_formation_quality(data);
        strength *= quality_score * 0.25; // 25% weight

        // 4. Trend Context
        if is_reversal {
            let trend_score = Self::evaluate_trend_context(data);
            strength *= trend_score * 0.15; // 15% weight
        }

        // 5. Candlestick Size Consistency
        let consistency_score = Self::evaluate_candle_consistency(data);
        strength *= consistency_score * 0.15; // 15% weight

        // Bound the strength between 0 and 1
        strength.max(0.0).min(1.0)
    }

    pub fn evaluate_volume_pattern(data: &[MarketData]) -> f64 {
        let avg_volume =
            data.iter().map(|d| d.volume.to_f64().unwrap()).sum::<f64>() / data.len() as f64;

        let recent_volumes: Vec<f64> = data
            .iter()
            .take(3) // Look at most recent 3 candles
            .map(|d| d.volume.to_f64().unwrap())
            .collect();

        let recent_avg = recent_volumes.iter().sum::<f64>() / recent_volumes.len() as f64;
        let volume_increase = recent_avg / avg_volume;

        // Score based on volume increase
        if volume_increase >= 2.0 {
            1.0
        }
        // 200% or more of average volume
        else if volume_increase >= 1.5 {
            0.8
        }
        // 150% of average volume
        else if volume_increase >= 1.2 {
            0.6
        }
        // 120% of average volume
        else if volume_increase >= 1.0 {
            0.4
        }
        // Average volume
        else {
            0.2
        } // Below average volume
    }

    pub fn evaluate_formation_quality(data: &[MarketData]) -> f64 {
        let mut quality_score = 1.0;

        // Check price levels alignment
        let price_alignment = Self::check_price_levels_alignment(data);
        quality_score *= price_alignment;

        // Check time symmetry
        let time_symmetry = Self::check_time_symmetry(data);
        quality_score *= time_symmetry;

        // Check for noise (false breakouts, spikes)
        let noise_factor = Self::calculate_noise_factor(data);
        quality_score *= noise_factor;

        quality_score
    }

    pub fn check_price_levels_alignment(data: &[MarketData]) -> f64 {
        let highs: Vec<f64> = data.iter().map(|d| d.high.to_f64().unwrap()).collect();

        let lows: Vec<f64> = data.iter().map(|d| d.low.to_f64().unwrap()).collect();

        // Calculate standard deviation of pivots
        let high_std = Self::calculate_std_dev(&highs);
        let low_std = Self::calculate_std_dev(&lows);

        // Lower standard deviation means better alignment
        let alignment_score =
            1.0 - (high_std + low_std) / (highs.iter().sum::<f64>() / highs.len() as f64);

        alignment_score.max(0.0).min(1.0)
    }

    pub fn check_time_symmetry(data: &[MarketData]) -> f64 {
        if data.len() < 3 {
            return 0.5;
        }

        // Find the middle point of the pattern
        let mid_point = data.len() / 2;
        let left_side = &data[mid_point..];
        let right_side = &data[..mid_point];

        // Compare the time distances of key points
        let left_duration = left_side.len() as f64;
        let right_duration = right_side.len() as f64;

        (left_duration.min(right_duration) / left_duration.max(right_duration)).powf(0.5)
    }

    pub fn calculate_noise_factor(data: &[MarketData]) -> f64 {
        let closes: Vec<f64> = data.iter().map(|d| d.close.to_f64().unwrap()).collect();

        // Calculate price volatility
        let volatility = Self::calculate_std_dev(&closes);
        let avg_price = closes.iter().sum::<f64>() / closes.len() as f64;
        let normalized_volatility = volatility / avg_price;

        // Less noise = higher score
        let noise_score = 1.0 - normalized_volatility;
        noise_score.max(0.2).min(1.0) // Never go below 0.2
    }

    pub fn evaluate_trend_context(data: &[MarketData]) -> f64 {
        let mut trend_score = 1.0;

        // Calculate trend strength using ADX
        let adx = Self::calculate_adx(data, 14);
        let adx_score = adx / 100.0;
        trend_score *= adx_score;

        // Check trend duration
        let trend_duration_score = Self::evaluate_trend_duration(data);
        trend_score *= trend_duration_score;

        // Check trend momentum
        let momentum_score = Self::evaluate_trend_momentum(data);
        trend_score *= momentum_score;

        trend_score
    }

    pub fn evaluate_trend_duration(data: &[MarketData]) -> f64 {
        let ideal_duration = 20; // Ideal number of candles for trend
        let actual_duration = data.len();

        let duration_ratio = actual_duration as f64 / ideal_duration as f64;
        if duration_ratio >= 0.8 && duration_ratio <= 1.5 {
            1.0
        } else if duration_ratio > 1.5 {
            0.8 // Longer trends are still good but slightly penalized
        } else {
            0.5 + (duration_ratio * 0.5) // Shorter trends are more heavily penalized
        }
    }

    pub fn evaluate_trend_momentum(data: &[MarketData]) -> f64 {
        if data.len() < 2 {
            return 0.5;
        }

        let closes: Vec<f64> = data.iter().map(|d| d.close.to_f64().unwrap()).collect();

        let roc = (closes[0] - closes[closes.len() - 1]) / closes[closes.len() - 1];
        let abs_roc = roc.abs();

        if abs_roc >= 0.05 {
            1.0
        }
        // Strong momentum
        else if abs_roc >= 0.03 {
            0.8
        }
        // Good momentum
        else if abs_roc >= 0.01 {
            0.6
        }
        // Moderate momentum
        else {
            0.4
        } // Weak momentum
    }

    pub fn evaluate_candle_consistency(data: &[MarketData]) -> f64 {
        let body_sizes: Vec<f64> = data
            .iter()
            .map(|d| (d.close - d.open).abs().to_f64().unwrap())
            .collect();

        let shadow_sizes: Vec<f64> = data
            .iter()
            .map(|d| {
                let upper = d.high - d.close.max(d.open);
                let lower = d.close.min(d.open) - d.low;
                (upper + lower).to_f64().unwrap()
            })
            .collect();

        let body_consistency = 1.0
            - (Self::calculate_std_dev(&body_sizes)
                / (body_sizes.iter().sum::<f64>() / body_sizes.len() as f64));

        let shadow_consistency = 1.0
            - (Self::calculate_std_dev(&shadow_sizes)
                / (shadow_sizes.iter().sum::<f64>() / shadow_sizes.len() as f64));

        let consistency_score = (body_consistency + shadow_consistency) / 2.0;
        consistency_score.clamp(0.0, 1.0)
    }
}

#[derive(Error, Debug)]
pub enum WorkerError {
    #[error("Market data error: {0}")]
    MarketData(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

impl std::convert::From<Box<dyn std::error::Error>> for WorkerError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        WorkerError::MarketData(error.to_string())
    }
}

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
}

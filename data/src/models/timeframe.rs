use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct TimeFrame {
    pub id: Uuid,

    #[validate(length(min = 1, max = 10))]
    pub name: String,

    pub interval_minutes: i32,
    pub is_enabled: bool,
    pub weight: Decimal,
    pub created_at: DateTime<Utc>,
}

impl TimeFrame {
    pub fn new(
        name: String,
        interval_minutes: i32,
        weight: Decimal,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            interval_minutes,
            is_enabled: true,
            weight,
            created_at: Utc::now(),
        }
    }
}


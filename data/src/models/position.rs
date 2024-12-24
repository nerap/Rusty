// models/market_data.rs
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Position {
    pub id: Uuid,
    pub market_data_id: Uuid,

    #[validate(length(min = 1, max = 20))]
    pub symbol: String,

    #[validate(length(min = 1, max = 10))]
    pub contract_type: String,

    #[validate(length(min = 1, max = 5))]
    pub side: String,

    pub size: Decimal,
    pub entry_price: Decimal,
    pub take_profit: Option<Decimal>,
    pub stop_loss: Option<Decimal>,
    pub entry_time: DateTime<Utc>,
    pub exit_time: Option<DateTime<Utc>>,
    pub exit_price: Option<Decimal>,
    pub pnl: Option<Decimal>,

    #[validate(length(min = 1, max = 10))]
    pub status: String,

    pub created_at: DateTime<Utc>,
}

impl Position {
    pub fn new(
        market_data_id: Uuid,
        symbol: String,
        contract_type: String,
        side: String,
        size: Decimal,
        entry_price: Decimal,
        entry_time: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            market_data_id,
            symbol,
            contract_type,
            side,
            size,
            entry_price,
            take_profit: None,
            stop_loss: None,
            entry_time,
            exit_time: None,
            exit_price: None,
            pnl: None,
            status: "open".to_string(),
            created_at: Utc::now(),
        }
    }
}


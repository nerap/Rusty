// models/market_data.rs
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ModelPrediction {
    pub id: Uuid,
    pub market_data_id: Uuid,
    pub timeframe_id: Uuid,
    pub lstm_pred: Decimal,
    pub cnn_pred: Decimal,
    pub dnn_pred: Decimal,
    pub ensemble_pred: Decimal,

    #[validate(custom = "validate_confidence")]
    pub confidence: Decimal,

    pub prediction_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl ModelPrediction {
    pub fn new(
        market_data_id: Uuid,
        timeframe_id: Uuid,
        lstm_pred: Decimal,
        cnn_pred: Decimal,
        dnn_pred: Decimal,
        ensemble_pred: Decimal,
        confidence: Decimal,
        prediction_time: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            market_data_id,
            timeframe_id,
            lstm_pred,
            cnn_pred,
            dnn_pred,
            ensemble_pred,
            confidence,
            prediction_time,
            created_at: Utc::now(),
        }
    }
}


fn validate_confidence(confidence: &Decimal) -> Result<(), validator::ValidationError> {
    if *confidence >= Decimal::ZERO && *confidence <= Decimal::ONE {
        Ok(())
    } else {
        Err(validator::ValidationError::new("confidence_range"))
    }
}

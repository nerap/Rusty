use chrono::{DateTime, Utc};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_postgres::{Client, Error as PgError, Row};

use crate::models::kline::{Kline, KlineCreatePayload, Validate};

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] PgError),
    #[error("Not found")]
    NotFound,
    #[error("Invalid data: {0}")]
    InvalidData(String),
}

type Result<T> = std::result::Result<T, RepositoryError>;

pub struct KlineRepository {
    client: Arc<Mutex<Client>>,
}

impl KlineRepository {
    pub fn new(client: Arc<Mutex<Client>>) -> Self {
        Self { client }
    }

    fn row_to_kline(&self, row: &Row) -> Result<Kline> {
        Ok(Kline {
            id: row.try_get("id")?,
            symbol: row.try_get("symbol")?,
            contract_type: row.try_get("contract_type")?,
            open_time: row.try_get("open_time")?,
            open_price: row.try_get("open_price")?,
            high_price: row.try_get("high_price")?,
            low_price: row.try_get("low_price")?,
            close_price: row.try_get("close_price")?,
            volume: row.try_get("volume")?,
            base_asset_volume: row.try_get("base_asset_volume")?,
            number_of_trades: row.try_get("number_of_trades")?,
            taker_buy_volume: row.try_get("taker_buy_volume")?,
            taker_buy_base_asset_volume: row.try_get("taker_buy_base_asset_volume")?,

            // Price Action Indicators
            rsi_14: row.try_get("rsi_14")?,
            rsi_4: row.try_get("rsi_4")?,
            stoch_k_14: row.try_get("stoch_k_14")?,
            stoch_d_14: row.try_get("stoch_d_14")?,
            cci_20: row.try_get("cci_20")?,

            // Trend Indicators
            macd_12_26: row.try_get("macd_12_26")?,
            macd_signal_9: row.try_get("macd_signal_9")?,
            macd_histogram: row.try_get("macd_histogram")?,
            ema_9: row.try_get("ema_9")?,
            ema_20: row.try_get("ema_20")?,
            ema_50: row.try_get("ema_50")?,
            ema_200: row.try_get("ema_200")?,

            // Volatility Indicators
            bollinger_upper: row.try_get("bollinger_upper")?,
            bollinger_middle: row.try_get("bollinger_middle")?,
            bollinger_lower: row.try_get("bollinger_lower")?,
            atr_14: row.try_get("atr_14")?,
            keltner_upper: row.try_get("keltner_upper")?,
            keltner_middle: row.try_get("keltner_middle")?,
            keltner_lower: row.try_get("keltner_lower")?,

            // Volume Indicators
            obv: row.try_get("obv")?,
            mfi_14: row.try_get("mfi_14")?,
            vwap: row.try_get("vwap")?,
            cmf_20: row.try_get("cmf_20")?,

            // Market Context
            funding_rate: row.try_get("funding_rate")?,
            open_interest: row.try_get("open_interest")?,
            long_short_ratio: row.try_get("long_short_ratio")?,
            cvd: row.try_get("cvd")?,

            // Position Management
            current_position: row.try_get("current_position")?,
            position_entry_price: row.try_get("position_entry_price")?,
            position_size: row.try_get("position_size")?,
            position_pnl: row.try_get("position_pnl")?,
            position_entry_time: row.try_get("position_entry_time")?,

            // Prediction and Performance
            predicted_position: row.try_get("predicted_position")?,
            prediction_confidence: row.try_get("prediction_confidence")?,
            actual_profit_loss: row.try_get("actual_profit_loss")?,
            trade_executed: row.try_get("trade_executed")?,

            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn create(&self, payload: KlineCreatePayload) -> Result<Kline> {
        if let Err(e) = payload.validate() {
            return Err(RepositoryError::InvalidData(e));
        }

        let result = self
            .client
            .lock()
            .await
            .query_one(
                "INSERT INTO Kline (
                    symbol, contract_type, open_time, open_price, high_price,
                    low_price, close_price, volume, base_asset_volume,
                    number_of_trades, taker_buy_volume, taker_buy_base_asset_volume
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                RETURNING *",
                &[
                    &payload.symbol,
                    &payload.contract_type,
                    &payload.open_time,
                    &payload.open_price,
                    &payload.high_price,
                    &payload.low_price,
                    &payload.close_price,
                    &payload.volume,
                    &payload.base_asset_volume,
                    &payload.number_of_trades,
                    &payload.taker_buy_volume,
                    &payload.taker_buy_base_asset_volume,
                ],
            )
            .await;

        match result {
            Ok(row) => self.row_to_kline(&row),
            Err(error) => Err(RepositoryError::Database(error)),
        }
    }

    async fn update(&self, kline: Kline) -> Result<Kline> {
        let row = self
            .client
            .lock()
            .await
            .query_one(
                "UPDATE kline SET
                    rsi_14 = $1, macd_12_26 = $2, ema_20 = $3,
                    predicted_position = $4, prediction_confidence = $5,
                    current_position = $6, position_entry_price = $7,
                    position_pnl = $8, trade_executed = $9
                WHERE id = $10 AND open_time = $11
                RETURNING *",
                &[
                    &kline.rsi_14,
                    &kline.macd_12_26,
                    &kline.ema_20,
                    &kline.predicted_position,
                    &kline.prediction_confidence,
                    &kline.current_position,
                    &kline.position_entry_price,
                    &kline.position_pnl,
                    &kline.trade_executed,
                    &kline.id,
                    &kline.open_time,
                ],
            )
            .await?;

        self.row_to_kline(&row)
    }

    async fn insert_batch(&self, klines: Vec<KlineCreatePayload>) -> Result<Vec<Kline>> {
        // Start transaction
        let mut client = self.client.lock().await;

        let tx = client.transaction().await?;

        let mut inserted_klines = Vec::with_capacity(klines.len());

        for payload in klines {
            if let Err(e) = payload.validate() {
                tx.rollback().await?;
                return Err(RepositoryError::InvalidData(e));
            }

            let row = tx
                .query_one(
                    "INSERT INTO kline (
                        symbol, contract_type, open_time, open_price, high_price,
                        low_price, close_price, volume, base_asset_volume,
                        number_of_trades, taker_buy_volume, taker_buy_base_asset_volume
                    ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                    RETURNING *",
                    &[
                        &payload.symbol,
                        &payload.contract_type,
                        &payload.open_time,
                        &payload.open_price,
                        &payload.high_price,
                        &payload.low_price,
                        &payload.close_price,
                        &payload.volume,
                        &payload.base_asset_volume,
                        &payload.number_of_trades,
                        &payload.taker_buy_volume,
                        &payload.taker_buy_base_asset_volume,
                    ],
                )
                .await?;

            inserted_klines.push(self.row_to_kline(&row)?);
        }

        // Commit transaction
        tx.commit().await?;

        Ok(inserted_klines)
    }

    async fn fetch_by_id(&self, id: i64) -> Result<Kline> {
        let row = self
            .client
            .lock()
            .await
            .query_one("SELECT * FROM kline WHERE id = $1", &[&id])
            .await?;

        self.row_to_kline(&row)
    }

    async fn fetch_by_symbol_timerange(
        &self,
        symbol: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<Kline>> {
        let rows = self
            .client
            .lock()
            .await
            .query(
                "SELECT * FROM kline
                WHERE symbol = $1
                AND open_time >= $2
                AND open_time <= $3
                ORDER BY open_time ASC",
                &[&symbol, &start_time, &end_time],
            )
            .await?;

        rows.iter().map(|row| self.row_to_kline(row)).collect()
    }

    async fn fetch_latest_by_symbol(&self, symbol: &str, limit: i32) -> Result<Vec<Kline>> {
        let rows = self
            .client
            .lock()
            .await
            .query(
                "SELECT * FROM kline
                WHERE symbol = $1
                ORDER BY open_time DESC
                LIMIT $2",
                &[&symbol, &limit],
            )
            .await?;

        rows.iter().map(|row| self.row_to_kline(row)).collect()
    }
}

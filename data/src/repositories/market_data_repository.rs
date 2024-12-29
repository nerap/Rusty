use std::sync::Arc;

use chrono::{DateTime, Utc};
use log::error;
use tokio::sync::Mutex;
use tokio_postgres::error::Error as PgError;
use tokio_postgres::Client;
use uuid::Uuid;

use crate::models::market_data::{MarketData, MarketDataIndicatorUpdate};

#[derive(Debug, thiserror::Error)]
pub enum MarketDataRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] PgError),
}

type Result<T> = std::result::Result<T, MarketDataRepositoryError>;

pub struct MarketDataRepository {
    client: Arc<Mutex<Client>>,
}

impl MarketDataRepository {
    pub fn new(client: Client) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    pub async fn create_batch(&self, data: &[MarketData]) -> Result<Vec<Uuid>> {
        let mut ids = Vec::with_capacity(data.len());
        let mut client = self.client.lock().await;
        let transaction = client.transaction().await?;

        for record in data {
            if record.close_time > Utc::now() {
                continue;
            }
            let row = transaction
                .query_one(
                    "INSERT INTO MarketData (
                        timeframe_id, symbol, contract_type, open_time, close_time,
                        open, high, low, close, volume, trades,
                        rsi_14, macd_line, macd_signal, macd_histogram,
                        bb_upper, bb_middle, bb_lower, atr_14, depth_imbalance,
                        volatility_1h, volatility_24h,
                        price_change_1h, price_change_24h,
                        volume_change_1h, volume_change_24h
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                            $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24,
                            $25, $26)
                    ON CONFLICT (open_time, timeframe_id) DO NOTHING
                    RETURNING id",
                    &[
                        &record.timeframe_id,
                        &record.symbol,
                        &record.contract_type,
                        &record.open_time,
                        &record.close_time,
                        &record.open,
                        &record.high,
                        &record.low,
                        &record.close,
                        &record.volume,
                        &record.trades,
                        &record.rsi_14,
                        &record.macd_line,
                        &record.macd_signal,
                        &record.macd_histogram,
                        &record.bb_upper,
                        &record.bb_middle,
                        &record.bb_lower,
                        &record.atr_14,
                        &record.depth_imbalance,
                        &record.volatility_1h,
                        &record.volatility_24h,
                        &record.price_change_1h,
                        &record.price_change_24h,
                        &record.volume_change_1h,
                        &record.volume_change_24h,
                    ],
                )
                .await;
            match row {
                Ok(row) => {
                    ids.push(row.get(0));
                    continue;
                }
                Err(e) if e.as_db_error().is_none() => continue,
                Err(e) => {
                    error!("{:?}", e);
                    continue;
                }
            }
        }

        transaction.commit().await?;
        Ok(ids)
    }

    pub async fn find_unanalyzed_market_data(&self, limit: i8) -> Result<Vec<MarketData>> {
        let rows= self
            .client
            .lock()
            .await
            .query(
                "SELECT * FROM MarketData WHERE analyzed = false AND close_time < NOW() ORDER BY open_time DESC LIMIT $1",
                &[&(limit as i64)],
            )
            .await;

        match rows {
            Ok(row) => Ok(row
                .iter()
                .map(|r| MarketData {
                    id: r.get(0),
                    timeframe_id: r.get(1),
                    symbol: r.get(2),
                    contract_type: r.get(3),
                    open_time: r.get(4),
                    close_time: r.get(5),
                    open: r.get(6),
                    high: r.get(7),
                    low: r.get(8),
                    close: r.get(9),
                    volume: r.get(10),
                    trades: r.get(11),
                    rsi_14: r.get(12),
                    macd_line: r.get(13),
                    macd_signal: r.get(14),
                    macd_histogram: r.get(15),
                    bb_upper: r.get(16),
                    bb_middle: r.get(17),
                    bb_lower: r.get(18),
                    atr_14: r.get(19),
                    depth_imbalance: r.get(20),
                    volatility_1h: r.get(21),
                    volatility_24h: r.get(22),
                    price_change_1h: r.get(23),
                    price_change_24h: r.get(24),
                    volume_change_1h: r.get(25),
                    volume_change_24h: r.get(26),
                    analyzed: r.get(27),
                    usable_by_model: r.get(28),
                    created_at: r.get(29),
                })
                .collect()),
            Err(error) => {
                error!("Error: {:?}", error);
                Err(MarketDataRepositoryError::Database(error))
            }
        }
    }

    pub async fn get_historical_data(
        &self,
        timeframe_id: Uuid,
        symbol: &str,
        contract_type: &str,
        from_time: DateTime<Utc>,
        record_count: i32,
    ) -> Result<Vec<MarketData>> {
        let rows = self
            .client
            .lock()
            .await
            .query(
                "SELECT * FROM MarketData
            WHERE timeframe_id = $1
            AND symbol = $2
            AND contract_type = $3
            AND open_time <= $4
            ORDER BY open_time DESC
            LIMIT $5",
                &[
                    &timeframe_id,
                    &symbol,
                    &contract_type,
                    &from_time,
                    &(record_count as i64),
                ],
            )
            .await;

        match rows {
            Ok(row) => Ok(row
                .iter()
                .map(|r| MarketData {
                    id: r.get(0),
                    timeframe_id: r.get(1),
                    symbol: r.get(2),
                    contract_type: r.get(3),
                    open_time: r.get(4),
                    close_time: r.get(5),
                    open: r.get(6),
                    high: r.get(7),
                    low: r.get(8),
                    close: r.get(9),
                    volume: r.get(10),
                    trades: r.get(11),
                    rsi_14: r.get(12),
                    macd_line: r.get(13),
                    macd_signal: r.get(14),
                    macd_histogram: r.get(15),
                    bb_upper: r.get(16),
                    bb_middle: r.get(17),
                    bb_lower: r.get(18),
                    atr_14: r.get(19),
                    depth_imbalance: r.get(20),
                    volatility_1h: r.get(21),
                    volatility_24h: r.get(22),
                    price_change_1h: r.get(23),
                    price_change_24h: r.get(24),
                    volume_change_1h: r.get(25),
                    volume_change_24h: r.get(26),
                    analyzed: r.get(27),
                    usable_by_model: r.get(28),
                    created_at: r.get(29),
                })
                .collect()),
            Err(error) => {
                error!("Error: {:?}", error);
                Err(MarketDataRepositoryError::Database(error))
            }
        }
    }

    pub async fn update_indicators(&self, update: MarketDataIndicatorUpdate) -> Result<()> {
        let client = self.client.lock().await;
        let rows = client
            .execute(
                "UPDATE MarketData SET
                   rsi_14 = $2,
                   macd_line = $3,
                   macd_signal = $4,
                   macd_histogram = $5,
                   bb_upper = $6,
                   bb_middle = $7,
                   bb_lower = $8,
                   atr_14 = $9,
                   depth_imbalance = $10,
                   volatility_1h = $11,
                   volatility_24h = $12,
                   price_change_1h = $13,
                   price_change_24h = $14,
                   volume_change_1h = $15,
                   volume_change_24h = $16,
                   analyzed = $17,
                   usable_by_model = $18
                WHERE id = $1",
                &[
                    &update.id,
                    &update.rsi_14,
                    &update.macd_line,
                    &update.macd_signal,
                    &update.macd_histogram,
                    &update.bb_upper,
                    &update.bb_middle,
                    &update.bb_lower,
                    &update.atr_14,
                    &update.depth_imbalance,
                    &update.volatility_1h,
                    &update.volatility_24h,
                    &update.price_change_1h,
                    &update.price_change_24h,
                    &update.volume_change_1h,
                    &update.volume_change_24h,
                    &update.analyzed,
                    &update.usable_by_model,
                ],
            )
            .await;
        match rows {
            Ok(_rows) => Ok(()),
            Err(error) => {
                println!("Error: {:?}", error);
                Err(MarketDataRepositoryError::Database(error))
            }
        }
    }

    pub async fn find_latest_by_timeframe(
        &self,
        timeframe_id: &Uuid,
    ) -> Result<Option<MarketData>> {
        let row = self
            .client
            .lock()
            .await
            .query_opt(
                "SELECT * FROM MarketData
                WHERE timeframe_id = $1
                ORDER BY open_time DESC
                LIMIT 1",
                &[timeframe_id],
            )
            .await?;

        Ok(row.map(|r| MarketData {
            id: r.get(0),
            timeframe_id: r.get(1),
            symbol: r.get(2),
            contract_type: r.get(3),
            open_time: r.get(4),
            close_time: r.get(5),
            open: r.get(6),
            high: r.get(7),
            low: r.get(8),
            close: r.get(9),
            volume: r.get(10),
            trades: r.get(11),
            rsi_14: r.get(12),
            macd_line: r.get(13),
            macd_signal: r.get(14),
            macd_histogram: r.get(15),
            bb_upper: r.get(16),
            bb_middle: r.get(17),
            bb_lower: r.get(18),
            atr_14: r.get(19),
            depth_imbalance: r.get(20),
            volatility_1h: r.get(21),
            volatility_24h: r.get(22),
            price_change_1h: r.get(23),
            price_change_24h: r.get(24),
            volume_change_1h: r.get(25),
            volume_change_24h: r.get(26),
            analyzed: r.get(27),
            usable_by_model: r.get(28),
            created_at: r.get(29),
        }))
    }
}

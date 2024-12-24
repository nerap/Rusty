use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::Mutex;
use tokio_postgres::{Client, Error as PgError};
use uuid::Uuid;

use crate::models::market_data::{MarketData, MarketDataIndicatorUpdate};

#[derive(Debug, thiserror::Error)]
pub enum MarketDataRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] PgError),
    #[error("Not found")]
    NotFound,
    #[error("Invalid data: {0}")]
    InvalidData(String),
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

    pub async fn create(&self, data: &MarketData) -> Result<Uuid> {
        let row = self
            .client
            .lock()
            .await
            .query_one(
                "INSERT INTO MarketData (
                timeframe_id, symbol, contract_type, open_time, close_time,
                open, high, low, close, volume, trades,
                rsi_14, macd_line, macd_signal, macd_histogram,
                bb_upper, bb_middle, bb_lower, atr_14,
                bid_ask_spread, depth_imbalance, funding_rate, open_interest,
                long_short_ratio, volatility_1h, volatility_24h,
                price_change_1h, price_change_24h,
                volume_change_1h, volume_change_24h
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                    $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24,
                    $25, $26, $27, $28, $29, $30)
            RETURNING id",
                &[
                    &data.timeframe_id,
                    &data.symbol,
                    &data.contract_type,
                    &data.open_time,
                    &data.close_time,
                    &data.open,
                    &data.high,
                    &data.low,
                    &data.close,
                    &data.volume,
                    &data.trades,
                    &data.rsi_14,
                    &data.macd_line,
                    &data.macd_signal,
                    &data.macd_histogram,
                    &data.bb_upper,
                    &data.bb_middle,
                    &data.bb_lower,
                    &data.atr_14,
                    &data.bid_ask_spread,
                    &data.depth_imbalance,
                    &data.funding_rate,
                    &data.open_interest,
                    &data.long_short_ratio,
                    &data.volatility_1h,
                    &data.volatility_24h,
                    &data.price_change_1h,
                    &data.price_change_24h,
                    &data.volume_change_1h,
                    &data.volume_change_24h,
                ],
            )
            .await?;

        Ok(row.get(0))
    }

    pub async fn create_batch(&self, data: &[MarketData]) -> Result<Vec<Uuid>> {
        let mut ids = Vec::with_capacity(data.len());
        let mut client = self.client.lock().await;
        let transaction = client.transaction().await?;

        for record in data {
            let row = transaction
                .query_one(
                    "INSERT INTO MarketData (
                    timeframe_id, symbol, contract_type, open_time, close_time,
                    open, high, low, close, volume, trades,
                    rsi_14, macd_line, macd_signal, macd_histogram,
                    bb_upper, bb_middle, bb_lower, atr_14,
                    bid_ask_spread, depth_imbalance, funding_rate, open_interest,
                    long_short_ratio, volatility_1h, volatility_24h,
                    price_change_1h, price_change_24h,
                    volume_change_1h, volume_change_24h
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
                        $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24,
                        $25, $26, $27, $28, $29, $30)
                ON CONFLICT (open_time, timeframe_id, symbol, contract_type)
                DO UPDATE SET
                    open = EXCLUDED.open,
                    high = EXCLUDED.high,
                    low = EXCLUDED.low,
                    close = EXCLUDED.close,
                    volume = EXCLUDED.volume,
                    trades = EXCLUDED.trades
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
                        &record.bid_ask_spread,
                        &record.depth_imbalance,
                        &record.funding_rate,
                        &record.open_interest,
                        &record.long_short_ratio,
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
                Err(error) => {
                    println!("Error: {:?}", error);
                }
            }
        }

        transaction.commit().await?;
        Ok(ids)
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<MarketData>> {
        let row = self
            .client
            .lock()
            .await
            .query_opt("SELECT * FROM MarketData WHERE id = $1", &[&id])
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
            bid_ask_spread: r.get(20),
            depth_imbalance: r.get(21),
            funding_rate: r.get(22),
            open_interest: r.get(23),
            long_short_ratio: r.get(24),
            volatility_1h: r.get(25),
            volatility_24h: r.get(26),
            price_change_1h: r.get(27),
            price_change_24h: r.get(28),
            volume_change_1h: r.get(29),
            volume_change_24h: r.get(30),
            analyzed: r.get(31),
            usable_by_model: r.get(32),
            created_at: r.get(33),
        }))
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
                    bid_ask_spread: r.get(20),
                    depth_imbalance: r.get(21),
                    funding_rate: r.get(22),
                    open_interest: r.get(23),
                    long_short_ratio: r.get(24),
                    volatility_1h: r.get(25),
                    volatility_24h: r.get(26),
                    price_change_1h: r.get(27),
                    price_change_24h: r.get(28),
                    volume_change_1h: r.get(29),
                    volume_change_24h: r.get(30),
                    analyzed: r.get(31),
                    usable_by_model: r.get(32),
                    created_at: r.get(33),
                })
                .collect()),
            Err(error) => {
                println!("Error: {:?}", error);
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
                    bid_ask_spread: r.get(20),
                    depth_imbalance: r.get(21),
                    funding_rate: r.get(22),
                    open_interest: r.get(23),
                    long_short_ratio: r.get(24),
                    volatility_1h: r.get(25),
                    volatility_24h: r.get(26),
                    price_change_1h: r.get(27),
                    price_change_24h: r.get(28),
                    volume_change_1h: r.get(29),
                    volume_change_24h: r.get(30),
                    analyzed: r.get(31),
                    usable_by_model: r.get(32),
                    created_at: r.get(33),
                })
                .collect()),
            Err(error) => {
                println!("Error: {:?}", error);
                Err(MarketDataRepositoryError::Database(error))
            }
        }
    }

    pub async fn update_indicators(&self, update: MarketDataIndicatorUpdate) -> Result<()> {
        let client = self.client.lock().await;
        let rows = client
            .execute(
                "UPDATE MarketData SET
                   rsi_14 = $2, macd_line = $3, macd_signal = $4, macd_histogram = $5,
                   bb_upper = $6, bb_middle = $7, bb_lower = $8, atr_14 = $9,
                   depth_imbalance = $10, volatility_24h = $11,
                   analyzed = $12, usable_by_model = $13
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
                    &update.volatility_24h,
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
}

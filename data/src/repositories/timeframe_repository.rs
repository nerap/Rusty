use anyhow::Result;
use rust_decimal::Decimal;
use tokio_postgres::Client;
use uuid::Uuid;

use crate::{
    lib::helper::Helper,
    models::timeframe::{ContractType, TimeFrame},
};

pub struct TimeFrameRepository {
    client: Client,
}

impl TimeFrameRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn create(&self, time_frame: &TimeFrame) -> Result<TimeFrame> {
        let row = self
            .client
            .query_one(
                "INSERT INTO Timeframes (symbol, contract_type, interval_minutes, weight)
                    VALUES ($1, $2, $3,$4)
                 RETURNING *",
                &[
                    &time_frame.symbol,
                    &time_frame.contract_type,
                    &time_frame.interval_minutes,
                    &time_frame.weight,
                ],
            )
            .await?;

        Ok(TimeFrame {
            id: row.get(0),
            symbol: row.get(1),
            contract_type: row.get(2),
            interval_minutes: row.get(3),
            weight: row.get(4),
            created_at: row.get(5),
        })
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<TimeFrame>> {
        let row = self
            .client
            .query_opt("SELECT * FROM Timeframes WHERE id = $1", &[&id])
            .await?;

        Ok(row.map(|r| TimeFrame {
            id: r.get(0),
            symbol: r.get(1),
            contract_type: r.get(2),
            interval_minutes: r.get(3),
            weight: r.get(4),
            created_at: r.get(5),
        }))
    }

    pub async fn find_or_create(
        &self,
        symbol: String,
        contract_type: ContractType,
        interval: String,
        weight: Decimal,
    ) -> Result<TimeFrame> {
        let interval_minutes = Helper::interval_to_minutes(&interval).unwrap();

        if let Some(row) = self
            .client
            .query_opt(
                "SELECT id,
                        symbol,
                        contract_type,
                        interval_minutes,
                        weight,
                        created_at
                 FROM Timeframes
                 WHERE symbol = $1
                   AND contract_type = $2
                   AND interval_minutes = $3
                   AND weight = $4",
                &[&symbol, &contract_type, &interval_minutes, &weight],
            )
            .await?
        {
            return Ok(TimeFrame {
                id: row.get(0),
                symbol: row.get(1),
                contract_type: row.get(2),
                interval_minutes: row.get(3),
                weight: row.get(4),
                created_at: row.get(5),
            });
        }

        let timeframe = TimeFrame::new(symbol, contract_type, interval_minutes, weight);

        self.create(&timeframe).await
    }
}

use rust_decimal::Decimal;
use tokio_postgres::Client;
use uuid::Uuid;
use anyhow::Result;

use crate::models::timeframe::TimeFrame;

pub struct TimeFrameRepository {
    client: Client,
}

impl TimeFrameRepository {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn create(&self, time_frame: &TimeFrame) -> Result<Uuid> {
        let row = self.client.query_one(
            "INSERT INTO Timeframes (name, interval_minutes, is_enabled, weight)
            VALUES ($1, $2, $3, $4)
            RETURNING id",
            &[
                &time_frame.name,
                &time_frame.interval_minutes,
                &time_frame.is_enabled,
                &time_frame.weight,
            ],
        ).await?;

        Ok(row.get(0))
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<TimeFrame>> {
        let row = self.client.query_opt(
            "SELECT * FROM Timeframes WHERE id = $1",
            &[&id]
        ).await?;

        Ok(row.map(|r| TimeFrame {
            id: r.get(0),
            name: r.get(1),
            interval_minutes: r.get(2),
            is_enabled: r.get(3),
            weight: r.get(4),
            created_at: r.get(5),
        }))
    }

    pub async fn find_by_interval_and_weight(
        &self,
        interval_minutes: i32,
        weight: Decimal
    ) -> Result<Option<TimeFrame>> {
        let row = self.client.query_opt(
            "SELECT * FROM Timeframes
            WHERE interval_minutes = $1
            AND weight = $2",
            &[&interval_minutes, &weight]
        ).await?;

        Ok(row.map(|r| TimeFrame {
            id: r.get(0),
            name: r.get(1),
            interval_minutes: r.get(2),
            is_enabled: r.get(3),
            weight: r.get(4),
            created_at: r.get(5),
        }))
    }

    pub async fn find_all_enabled(&self) -> Result<Vec<TimeFrame>> {
        let rows = self.client.query(
            "SELECT * FROM Timeframes
            WHERE is_enabled = true
            ORDER BY interval_minutes",
            &[]
        ).await?;

        Ok(rows.iter().map(|r| TimeFrame {
            id: r.get(0),
            name: r.get(1),
            interval_minutes: r.get(2),
            is_enabled: r.get(3),
            weight: r.get(4),
            created_at: r.get(5),
        }).collect())
    }
}

use std::sync::Arc;

use crate::repositories::kline_repostory::KlineRepository;
use tokio_postgres::NoTls;
use tokio::sync::Mutex;

pub struct DatabaseService {
    pub kline: KlineRepository,
}

impl DatabaseService {
    pub async fn new() -> Result<Self, tokio_postgres::Error> {
        let (client, connection) = tokio_postgres::connect(
            "host=localhost dbname=rusty user=admin password=admin",
            NoTls,
        )
        .await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Self {
            kline: KlineRepository::new(Arc::new(Mutex::new(client))),
        })
    }
}

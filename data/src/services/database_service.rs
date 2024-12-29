use std::env;

use tokio_postgres::{Client, NoTls};

pub struct DatabaseService {
    pub client: Client
}

impl DatabaseService {
    pub async fn new() -> Result<Self, tokio_postgres::Error> {
        let host = env::var("DB_HOST").unwrap_or_else(|_| "timescaledb".to_string());
        let user = env::var("DB_USER").unwrap_or_else(|_| "admin".to_string());
        let password = env::var("DB_PASSWORD").unwrap_or_else(|_| "admin".to_string());
        let dbname = env::var("DB_NAME").unwrap_or_else(|_| "rusty".to_string());
        let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let connection_string = format!(
            "host={} dbname={} user={} password={} port={}",
            host, dbname, user, password, port
        );

        tracing::info!("Attempting database connection...");

        let result = tokio_postgres::connect(&connection_string, NoTls).await;
        match result {
            Ok((client, connection)) => {
                tracing::info!("Database connected successfully");

                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        tracing::error!("connection error: {}", e);
                    }
                });

                Ok(Self {
                    client
                })
            },
            Err(error) => {
                tracing::error!("connection error: {}", error);
                Err(error)

            }
        }
    }
}

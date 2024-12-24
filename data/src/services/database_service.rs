use tokio_postgres::{Client, NoTls};

pub struct DatabaseService {}

impl DatabaseService {
    pub async fn new() -> Result<Client, tokio_postgres::Error> {
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

        Ok(client)
    }
}

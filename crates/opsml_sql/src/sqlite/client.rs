use crate::base::SqlClient;
use async_trait::async_trait;
use opsml_settings::config::OpsmlDatabaseSettings;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

pub struct SqliteClient {
    pub pool: Pool<Sqlite>,
}

#[async_trait]
impl SqlClient for SqliteClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self {
        let pool = SqlitePoolOptions::new()
            .max_connections(settings.max_connections)
            .connect(&settings.connection_uri)
            .await
            .expect("Failed to connect to SQLite database");

        Self { pool }
    }

    async fn run_migration(&self, migration: &str) -> Result<(), sqlx::Error> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .expect("Failed to run migration");

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_sqlite() -> () {
        let config = OpsmlDatabaseSettings {
            connection_uri: "sqlite::memory:".to_string(),
            max_connections: 1,
        };

        let client = SqliteClient::new(&config).await;
    }
}

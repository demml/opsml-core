use crate::base::SqlClient;
use async_trait::async_trait;
use opsml_settings::config::OpsmlDatabaseSettings;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

pub struct SqlLiteClient {
    pub pool: Pool<Sqlite>,
}

#[async_trait]
impl SqlClient for SqlLiteClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self {
        let pool = SqlitePoolOptions::new()
            .max_connections(settings.max_connections)
            .connect(&settings.connection_uri)
            .await
            .expect("Failed to connect to SQLite database");

        Self { pool }
    }
}

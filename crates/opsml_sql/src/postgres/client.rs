use crate::base::SqlClient;
use async_trait::async_trait;
use opsml_settings::config::OpsmlDatabaseSettings;
use sqlx::{
    postgres::{PgPoolOptions, Postgres},
    Pool,
};

pub struct SqlLiteClient {
    pub pool: Pool<Postgres>,
}

#[async_trait]
impl SqlClient for SqlLiteClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(settings.max_connections)
            .connect(&settings.connection_uri)
            .await
            .expect("Failed to connect to Postgres database");

        Self { pool }
    }
}

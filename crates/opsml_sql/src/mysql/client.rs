use crate::base::SqlClient;
use async_trait::async_trait;
use opsml_settings::config::OpsmlDatabaseSettings;
use sqlx::{
    mysql::{MySql, MySqlPoolOptions},
    Pool,
};

pub struct SqlLiteClient {
    pub pool: Pool<MySql>,
}

#[async_trait]
impl SqlClient for SqlLiteClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self {
        let pool = MySqlPoolOptions::new()
            .max_connections(settings.max_connections)
            .connect(&settings.connection_uri)
            .await
            .expect("Failed to connect to Postgres database");

        Self { pool }
    }
}

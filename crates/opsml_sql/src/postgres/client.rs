use crate::base::SqlClient;
use async_trait::async_trait;
use opsml_settings::config::OpsmlDatabaseSettings;
use sqlx::{
    postgres::{PgPoolOptions, Postgres},
    Pool,
};

pub struct PostgresClient {
    pub pool: Pool<Postgres>,
}

#[async_trait]
impl SqlClient for PostgresClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(settings.max_connections)
            .connect(&settings.connection_uri)
            .await
            .expect("Failed to connect to Postgres database");

        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_postgres() {
        let config = OpsmlDatabaseSettings {
            connection_uri: env::var("OPSML_TRACKING_URI")
                .unwrap_or_else(|_| "postgres://admin:admin@localhost:5432/testdb".to_string()),
            max_connections: 1,
        };

        let client = PostgresClient::new(&config).await;
        // Add assertions or further test logic here
    }
}

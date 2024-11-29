use crate::base::SqlClient;
use async_trait::async_trait;
use opsml_error::error::SqlError;
use opsml_logging::logging::setup_logging;
use opsml_settings::config::OpsmlDatabaseSettings;
use sqlx::{
    mysql::{MySql, MySqlPoolOptions},
    Pool,
};
use tracing::info;

pub struct MySqlClient {
    pub pool: Pool<MySql>,
}

#[async_trait]
impl SqlClient for MySqlClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self {
        let pool = MySqlPoolOptions::new()
            .max_connections(settings.max_connections)
            .connect(&settings.connection_uri)
            .await
            .expect("Failed to connect to Postgres database");

        // attempt to start logging, silently fail if it fails
        let _ = (setup_logging().await).is_ok();

        let client = Self { pool };

        // run migrations
        client
            .run_migrations()
            .await
            .expect("Failed to run migrations");

        client
    }
    async fn run_migrations(&self) -> Result<(), SqlError> {
        info!("Running migrations");
        sqlx::migrate!("src/mysql/migrations")
            .run(&self.pool)
            .await
            .map_err(|e| SqlError::MigrationError(format!("{}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use opsml_settings::config::SqlType;
    use std::env;

    #[tokio::test]
    async fn test_mysql() {
        let config = OpsmlDatabaseSettings {
            connection_uri: env::var("OPSML_TRACKING_URI")
                .unwrap_or_else(|_| "mysql://admin:admin@localhost:3306/testdb".to_string()),
            max_connections: 1,
            sql_type: SqlType::MySql,
        };

        let _client = MySqlClient::new(&config).await;
    }
}

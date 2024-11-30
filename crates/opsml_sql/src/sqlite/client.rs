use crate::base::CardSQLTableNames;
use crate::base::SqlClient;
use crate::queries::shared::Queries;
use crate::sqlite::schema::VersionResult;
use async_trait::async_trait;
use opsml_error::error::SqlError;
use opsml_logging::logging::setup_logging;
use opsml_settings::config::OpsmlDatabaseSettings;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use tracing::info;

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
        sqlx::migrate!("src/sqlite/migrations")
            .run(&self.pool)
            .await
            .map_err(|e| SqlError::MigrationError(format!("{}", e)))?;

        Ok(())
    }

    async fn get_versions(
        &self,
        table: CardSQLTableNames,
        name: &str,
        repository: &str,
        version: Option<&str>,
    ) -> Result<(), SqlError> {
        let query = Queries::GetVersions.get_query();

        let built_query = if version.is_some() {
            let query = format!(
                "{} AND version like '%{}%' ORDER BY timestamp, version",
                query.sql,
                version.unwrap()
            );

            sqlx::query_as::<_, VersionResult>(&query)
                .bind(&table.to_string())
                .bind(&name)
                .bind(&repository)
                .bind(version.unwrap())
                .fetch_all(&self.pool)
                .await
        } else {
            let query = format!("{} ORDER BY timestamp, version", query.sql);
            sqlx::query_as::<_, VersionResult>(&query)
                .bind(&table.to_string())
                .bind(&name)
                .bind(&repository)
                .fetch_all(&self.pool)
                .await
        };

        //

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use opsml_settings::config::SqlType;

    #[tokio::test]
    async fn test_sqlite() {
        let config = OpsmlDatabaseSettings {
            connection_uri: "sqlite::memory:".to_string(),
            max_connections: 1,
            sql_type: SqlType::Sqlite,
        };

        let _client = SqliteClient::new(&config).await;
    }
}

use crate::base::CardSQLTableNames;
use crate::base::SqlClient;
use crate::schemas::schema::VersionResult;
use crate::sqlite::queries::helper::Queries;
use async_trait::async_trait;
use chrono::format;
use opsml_error::error::SqlError;
use opsml_logging::logging::setup_logging;
use opsml_settings::config::OpsmlDatabaseSettings;
use opsml_utils::semver::VersionParser;
use sqlx::sqlite::SqliteRow;
use sqlx::FromRow;
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
    ) -> Result<Vec<VersionResult>, SqlError> {
        // if version is None, get the latest version
        let cards = match version {
            Some(version) => {
                let version_bounds = VersionParser::get_version_to_search(version)
                    .map_err(|e| SqlError::VersionError(format!("{}", e)))?;

                let upper_bound = if version_bounds.no_upper_bound {
                    "".to_string()
                } else {
                    format!(
                        "AND (major = {} AND minor < {})",
                        version_bounds.upper_bound.major, version_bounds.upper_bound.minor,
                    )
                };

                let query = Queries::GetCardsWithVersion.get_query();
                let result: Vec<VersionResult> = sqlx::query_as(&query.sql)
                    .bind(table.to_string())
                    .bind(name)
                    .bind(repository)
                    .bind(version)
                    .bind(version_bounds.upper_bound.major.to_string())
                    .bind(version_bounds.upper_bound.minor.to_string())
                    .bind(upper_bound)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

                result
            }
            None => {
                let query = Queries::GetCardsWithoutVersion.get_query();
                let result: Vec<VersionResult> = sqlx::query_as(&query.sql)
                    .bind(table.to_string())
                    .bind(name)
                    .bind(repository)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

                result
            }
        };

        Ok(cards)

        //
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

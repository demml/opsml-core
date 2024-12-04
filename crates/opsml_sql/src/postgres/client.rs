use crate::base::CardSQLTableNames;
use crate::base::SqlClient;
use crate::schemas::schema::VersionResult;
use crate::sqlite::queries::helper::Queries;
use async_trait::async_trait;
use opsml_error::error::SqlError;
use opsml_logging::logging::setup_logging;
use opsml_settings::config::OpsmlDatabaseSettings;
use opsml_utils::semver::VersionParser;
use sqlx::{
    postgres::{PgPoolOptions, Postgres},
    Pool,
};
use tracing::info;

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
        sqlx::migrate!("src/postgres/migrations")
            .run(&self.pool)
            .await
            .map_err(|e| SqlError::MigrationError(format!("{}", e)))?;

        Ok(())
    }
    async fn get_versions(
        &self,
        table: CardSQLTableNames,
        name: Option<&str>,
        repository: Option<&str>,
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
    use std::env;

    #[tokio::test]
    async fn test_postgres() {
        let config = OpsmlDatabaseSettings {
            connection_uri: env::var("OPSML_TRACKING_URI")
                .unwrap_or_else(|_| "postgres://admin:admin@localhost:5432/testdb".to_string()),
            max_connections: 1,
            sql_type: SqlType::Postgres,
        };

        let _client = PostgresClient::new(&config).await;
        // Add assertions or further test logic here
    }
}

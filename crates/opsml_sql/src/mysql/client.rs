use crate::base::CardSQLTableNames;
use crate::base::SqlClient;
use crate::schemas::schema::VersionResult;
use async_trait::async_trait;
use opsml_error::error::SqlError;
use opsml_logging::logging::setup_logging;
use opsml_settings::config::OpsmlDatabaseSettings;
use opsml_utils::semver::VersionParser;
use opsml_utils::semver::VersionValidator;
use semver::Version;
use sqlx::{
    mysql::{MySql, MySqlPoolOptions},
    Execute, Pool, QueryBuilder,
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

    /// Primary query for retrieving versions from the database. Mainly used to get most recent version when determining version increment
    ///
    /// # Arguments
    ///
    /// * `table` - The table to query
    /// * `name` - The name of the card
    /// * `repository` - The repository of the card
    /// * `version` - The version of the card
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - A vector of strings representing the sorted (desc) versions of the card
    async fn get_versions(
        &self,
        table: CardSQLTableNames,
        name: &str,
        repository: &str,
        version: Option<&str>,
    ) -> Result<Vec<String>, SqlError> {
        // if version is None, get the latest version
        let query = "SELECT date, timestamp, name, repository, major, minor, patch, pre_tag, build_tag, contact, uid";

        let mut builder = QueryBuilder::<MySql>::new(query);
        builder.push(format!(" FROM {} ", table.to_string()));

        // add where clause due to multiple combinations
        builder.push(" WHERE 1==1");
        builder.push(" AND name == ");
        builder.push_bind(name);

        builder.push(" AND repository == ");
        builder.push_bind(repository);
        if let Some(version) = version {
            let version_bounds = VersionParser::get_version_to_search(version)
                .map_err(|e| SqlError::VersionError(format!("{}", e)))?;

            // construct lower bound (already validated)
            builder.push(format!(
                " AND (major >= {} AND minor >= {} and patch >= {})",
                version_bounds.lower_bound.major,
                version_bounds.lower_bound.minor,
                version_bounds.lower_bound.patch
            ));

            if !version_bounds.no_upper_bound {
                // construct upper bound based on number of components
                if version_bounds.num_parts == 1 {
                    builder.push(format!(
                        " AND (major < {})",
                        version_bounds.upper_bound.major
                    ));
                } else if version_bounds.num_parts == 2 {
                    builder.push(format!(
                        " AND (major == {} AND minor < {})",
                        version_bounds.upper_bound.major, version_bounds.upper_bound.minor
                    ));
                } else if version_bounds.num_parts == 3
                    && version_bounds.parser_type == VersionParser::Tilde
                {
                    builder.push(format!(
                        " AND (major == {} AND minor < {})",
                        version_bounds.upper_bound.major, version_bounds.upper_bound.minor
                    ));
                } else if version_bounds.num_parts == 3
                    && version_bounds.parser_type == VersionParser::Caret
                {
                    builder.push(format!(
                        " AND (major == {} AND minor < {})",
                        version_bounds.upper_bound.major, version_bounds.upper_bound.minor
                    ));
                } else {
                    builder.push(format!(
                        " AND (major == {} AND minor == {} AND patch < {})",
                        version_bounds.upper_bound.major,
                        version_bounds.upper_bound.minor,
                        version_bounds.upper_bound.patch
                    ));
                }
            }
        }

        // order by timestamp and limit 20
        builder.push(" ORDER BY timestamp DESC LIMIT 20;");
        let sql = builder.build().sql();

        let cards: Vec<VersionResult> = sqlx::query_as(&sql)
            .bind(name)
            .bind(repository)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

        let versions = cards
            .iter()
            .map(|c| {
                c.to_version()
                    .map_err(|e| SqlError::VersionError(format!("{}", e)))
            })
            .collect::<Result<Vec<Version>, SqlError>>()?;

        // sort semvers
        VersionValidator::sort_semver_versions(versions, true)
            .map_err(|e| SqlError::VersionError(format!("{}", e)))
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

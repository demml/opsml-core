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
use sqlx::{query_builder::QueryBuilder, Execute};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use tracing::info;

pub struct SqliteClient {
    pub pool: Pool<Sqlite>,
}

#[async_trait]
impl SqlClient for SqliteClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self {
        // if the connection uri is not in memory, create the file
        if !settings.connection_uri.contains(":memory:") {
            // check if the file exists
            if !std::path::Path::new(&settings.connection_uri).exists() {
                // strip "sqlite:" from the connection uri
                let uri = settings.connection_uri.replace("sqlite:", "");

                // create the file
                std::fs::File::create(&uri)
                    .map_err(|e| {
                        SqlError::FileError(format!(
                            "Failed to create SQLite file: {} with error: {}",
                            &uri, e
                        ))
                    })
                    .unwrap();
            }
        }

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
        name: Option<&str>,
        repository: Option<&str>,
        version: Option<&str>,
    ) -> Result<Vec<VersionResult>, SqlError> {
        // if version is None, get the latest version
        let query = Queries::GetCardsWithoutVersion.get_query();

        let mut builder = QueryBuilder::<Sqlite>::new(&query.sql);
        builder.push(format!(" FROM {} ", table.to_string()));

        if let Some(name) = name {
            builder.push(" WHERE 1==1 AND name == ");
            builder.push_bind(name);
        }

        if let Some(repository) = repository {
            builder.push(" AND repository == ");
            builder.push_bind(repository);
        }

        if let Some(version) = version {
            let version_bounds = VersionParser::get_version_to_search(version)
                .map_err(|e| SqlError::VersionError(format!("{}", e)))?;

            // construct lower bound
            builder.push(format!(" AND (major == ",));
            builder.push_bind(version_bounds.lower_bound.major as i32);

            builder.push(format!(" AND minor >= ",));
            builder.push_bind(version_bounds.lower_bound.minor as i32);
            builder.push(")");

            // AND minor >= {})

            if !version_bounds.no_upper_bound {
                builder.push(format!(" AND (major == ",));
                builder.push_bind(version_bounds.upper_bound.major as i32);

                builder.push(format!(" AND minor < ",));
                builder.push_bind(version_bounds.upper_bound.minor as i32);
                builder.push(")");
            }
            let sql = builder.build().sql();

            let cards: Vec<VersionResult> = sqlx::query_as(&sql)
                .bind(name)
                .bind(repository)
                .bind(version_bounds.lower_bound.major.to_string())
                .bind(version_bounds.lower_bound.minor.to_string())
                .bind(version_bounds.upper_bound.major.to_string())
                .bind(version_bounds.upper_bound.minor.to_string())
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

            Ok(cards)
        } else {
            let sql = builder.build().sql();
            let cards: Vec<VersionResult> = sqlx::query_as(&sql)
                .bind(name)
                .bind(repository)
                .fetch_all(&self.pool)
                .await
                .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

            Ok(cards)
        }

        //
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    pub fn cleanup() {
        // delete ./test.db if exists
        if std::path::Path::new("./test.db").exists() {
            std::fs::remove_file("./test.db").unwrap();
        }
    }
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

    // create test for non-memory sqlite

    #[tokio::test]
    async fn test_sqlite_file() {
        cleanup();

        let config = OpsmlDatabaseSettings {
            connection_uri: "sqlite:./test.db".to_string(),
            max_connections: 1,
            sql_type: SqlType::Sqlite,
        };

        let client = SqliteClient::new(&config).await;

        // Run the SQL script to populate the database
        let script = std::fs::read_to_string("tests/populate_test_data.sql").unwrap();
        sqlx::query(&script).execute(&client.pool).await.unwrap();

        // get versions (should return 1)
        let versions = client
            .get_versions(
                CardSQLTableNames::Data,
                Some("Data10"),
                Some("repo10"),
                Some("~3.0.0"),
            )
            .await
            .unwrap();
        assert_eq!(versions.len(), 1);

        // check start version
        //let versions = client
        //    .get_versions(CardSQLTableNames::Data, Some("Data10"), None, Some("1.0.0"))
        //    .await
        //    .unwrap();
        //
        // delete ./test.db
        cleanup();
    }
}

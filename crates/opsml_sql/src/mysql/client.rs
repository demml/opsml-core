use crate::base::CardSQLTableNames;
use crate::base::SqlClient;
use crate::queries::shared::SqlHelper;
use crate::schemas::arguments::CardQueryArgs;
use crate::schemas::schema::Card;
use crate::schemas::schema::{
    AuditCardRecord, DataCardRecord, ModelCardRecord, PipelineCardRecord, RunCardRecord,
};
use crate::schemas::schema::{CardResults, VersionResult};
use async_trait::async_trait;
use opsml_error::error::SqlError;
use opsml_logging::logging::setup_logging;
use opsml_settings::config::OpsmlDatabaseSettings;
use opsml_utils::semver::{VersionParser, VersionValidator};
use opsml_utils::utils::is_valid_uuid4;
use semver::Version;
use sqlx::{
    mysql::{MySql, MySqlPoolOptions},
    Execute, Pool, QueryBuilder,
};
use tracing::info;

fn add_version_bounds(builder: &mut QueryBuilder<MySql>, version: &str) -> Result<(), SqlError> {
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
        } else if version_bounds.num_parts == 2
            || version_bounds.num_parts == 3 && version_bounds.parser_type == VersionParser::Tilde
            || version_bounds.num_parts == 3 && version_bounds.parser_type == VersionParser::Caret
        {
            builder.push(format!(
                " AND (major = {} AND minor < {})",
                version_bounds.upper_bound.major, version_bounds.upper_bound.minor
            ));
        } else {
            builder.push(format!(
                " AND (major = {} AND minor = {} AND patch < {})",
                version_bounds.upper_bound.major,
                version_bounds.upper_bound.minor,
                version_bounds.upper_bound.patch
            ));
        }
    }
    Ok(())
}

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
        builder.push(format!(" FROM {} ", table));

        // add where clause due to multiple combinations
        builder.push(" WHERE 1=1");
        builder.push(" AND name = ");
        builder.push_bind(name);

        builder.push(" AND repository = ");
        builder.push_bind(repository);

        if let Some(version) = version {
            add_version_bounds(&mut builder, version)?;
        }

        // order by timestamp and limit 20
        builder.push(" ORDER BY timestamp DESC LIMIT 20;");
        let sql = builder.build().sql();

        let cards: Vec<VersionResult> = sqlx::query_as(sql)
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

    /// Query cards based on the query arguments
    ///
    /// # Arguments
    ///
    /// * `table` - The table to query
    /// * `query_args` - The query arguments
    ///
    /// # Returns
    ///
    /// * `CardResults` - The results of the query
    async fn query_cards(
        &self,
        table: CardSQLTableNames,
        query_args: &CardQueryArgs,
    ) -> Result<CardResults, SqlError> {
        let query = format!("SELECT * FROM {}", table);
        let mut builder = QueryBuilder::<MySql>::new(query);
        builder.push(" WHERE 1=1");

        // check for uid. If uid is present, we only return that card
        if query_args.uid.is_some() {
            // validate uid
            is_valid_uuid4(query_args.uid.as_ref().unwrap())
                .map_err(|e| SqlError::GeneralError(e.to_string()))?;

            builder.push(format!(" AND uid = '{}'", query_args.uid.as_ref().unwrap()));
        } else {
            // add where clause due to multiple combinations
            if query_args.name.is_some() {
                builder.push(format!(
                    " AND name = '{}'",
                    query_args.name.as_ref().unwrap()
                ));
            }

            if query_args.repository.is_some() {
                builder.push(format!(
                    " AND repository = '{}'",
                    query_args.repository.as_ref().unwrap()
                ));
            }

            if query_args.version.is_some() {
                add_version_bounds(&mut builder, query_args.version.as_ref().unwrap())?;
            }

            if query_args.max_date.is_some() {
                builder.push(format!(
                    " AND DATE(date) <= STR_TO_DATE('{}', '%Y-%m-%d')",
                    query_args.max_date.as_ref().unwrap()
                ));
            }

            if query_args.tags.is_some() {
                let tags = query_args.tags.as_ref().unwrap();
                for (key, value) in tags.iter() {
                    builder.push(format!(
                        " AND JSON_EXTRACT(tags, '$.{}') = '{}'",
                        key, value
                    ));
                }
            }

            if query_args.sort_by_timestamp.unwrap_or(false) {
                builder.push(" ORDER BY timestamp DESC");
            } else {
                // sort by major, minor, patch
                builder.push(" ORDER BY major DESC, minor DESC, patch DESC");
            }

            if query_args.limit.is_some() {
                builder.push(format!(" LIMIT {}", query_args.limit.unwrap()));
            }
        }

        let sql = builder.sql();

        match table {
            CardSQLTableNames::Data => {
                let card: Vec<DataCardRecord> = sqlx::query_as(sql)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

                return Ok(CardResults::Data(card));
            }
            CardSQLTableNames::Model => {
                let card: Vec<ModelCardRecord> = sqlx::query_as(sql)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

                return Ok(CardResults::Model(card));
            }
            CardSQLTableNames::Run => {
                let card: Vec<RunCardRecord> = sqlx::query_as(sql)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

                return Ok(CardResults::Run(card));
            }

            CardSQLTableNames::Audit => {
                let card: Vec<AuditCardRecord> = sqlx::query_as(sql)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

                return Ok(CardResults::Audit(card));
            }
            CardSQLTableNames::Pipeline => {
                let card: Vec<PipelineCardRecord> = sqlx::query_as(sql)
                    .fetch_all(&self.pool)
                    .await
                    .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

                return Ok(CardResults::Pipeline(card));
            }
            _ => {
                return Err(SqlError::QueryError(
                    "Invalid table name for query".to_string(),
                ));
            }
        }
    }

    async fn insert_card(&self, table: CardSQLTableNames, card: &Card) -> Result<(), SqlError> {
        let query = match table {
            CardSQLTableNames::Data => match card {
                Card::Data(data) => SqlHelper::get_datacard_insert_query(data),
                _ => {
                    return Err(SqlError::QueryError(
                        "Invalid card type for insert".to_string(),
                    ));
                }
            },
            CardSQLTableNames::Model => match card {
                Card::Model(model) => SqlHelper::get_modelcard_insert_query(model),
                _ => {
                    return Err(SqlError::QueryError(
                        "Invalid card type for insert".to_string(),
                    ));
                }
            },
            CardSQLTableNames::Run => match card {
                Card::Run(run) => SqlHelper::get_runcard_insert_query(run),
                _ => {
                    return Err(SqlError::QueryError(
                        "Invalid card type for insert".to_string(),
                    ));
                }
            },
            CardSQLTableNames::Audit => match card {
                Card::Audit(audit) => SqlHelper::get_auditcard_insert_query(audit),
                _ => {
                    return Err(SqlError::QueryError(
                        "Invalid card type for insert".to_string(),
                    ));
                }
            },
            CardSQLTableNames::Pipeline => match card {
                Card::Pipeline(pipeline) => SqlHelper::get_pipelinecard_insert_query(pipeline),
                _ => {
                    return Err(SqlError::QueryError(
                        "Invalid card type for insert".to_string(),
                    ));
                }
            },
            _ => {
                return Err(SqlError::QueryError(
                    "Invalid table name for insert".to_string(),
                ));
            }
        };

        sqlx::query(&query)
            .execute(&self.pool)
            .await
            .map_err(|e| SqlError::QueryError(format!("{}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use opsml_settings::config::SqlType;
    use std::env;

    pub async fn cleanup(pool: &Pool<MySql>) {
        sqlx::raw_sql(
            r#"
            DELETE 
            FROM opsml_data_registry;

            DELETE 
            FROM opsml_model_registry;

            DELETE
            FROM opsml_run_registry;

            DELETE
            FROM opsml_audit_registry;

            DELETE
            FROM opsml_pipeline_registry;
            "#,
        )
        .fetch_all(pool)
        .await
        .unwrap();
    }

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

    #[tokio::test]
    async fn test_mysql_versions() {
        let config = OpsmlDatabaseSettings {
            connection_uri: env::var("OPSML_TRACKING_URI")
                .unwrap_or_else(|_| "mysql://admin:admin@localhost:3306/testdb".to_string()),
            max_connections: 1,
            sql_type: SqlType::MySql,
        };

        let client = MySqlClient::new(&config).await;

        cleanup(&client.pool).await;

        // Run the SQL script to populate the database
        let script = std::fs::read_to_string("tests/populate_mysql_test.sql").unwrap();
        sqlx::raw_sql(&script).execute(&client.pool).await.unwrap();

        // query all versions
        // get versions (should return 1)
        let versions = client
            .get_versions(CardSQLTableNames::Data, "Data1", "repo1", None)
            .await
            .unwrap();
        assert_eq!(versions.len(), 10);

        // check star pattern
        let versions = client
            .get_versions(CardSQLTableNames::Data, "Data1", "repo1", Some("*"))
            .await
            .unwrap();
        assert_eq!(versions.len(), 10);

        let versions = client
            .get_versions(CardSQLTableNames::Data, "Data1", "repo1", Some("1.*"))
            .await
            .unwrap();
        assert_eq!(versions.len(), 4);

        let versions = client
            .get_versions(CardSQLTableNames::Data, "Data1", "repo1", Some("1.1.*"))
            .await
            .unwrap();
        assert_eq!(versions.len(), 2);

        // check tilde pattern
        let versions = client
            .get_versions(CardSQLTableNames::Data, "Data1", "repo1", Some("~1"))
            .await
            .unwrap();
        assert_eq!(versions.len(), 4);

        // check tilde pattern
        let versions = client
            .get_versions(CardSQLTableNames::Data, "Data1", "repo1", Some("~1.1"))
            .await
            .unwrap();
        assert_eq!(versions.len(), 2);

        // check tilde pattern
        let versions = client
            .get_versions(CardSQLTableNames::Data, "Data1", "repo1", Some("~1.1.1"))
            .await
            .unwrap();
        assert_eq!(versions.len(), 1);

        let versions = client
            .get_versions(CardSQLTableNames::Data, "Data1", "repo1", Some("^2.0.0"))
            .await
            .unwrap();
        assert_eq!(versions.len(), 2);

        cleanup(&client.pool).await;
    }

    #[tokio::test]
    async fn test_mysql_query_cards() {
        let config = OpsmlDatabaseSettings {
            connection_uri: env::var("OPSML_TRACKING_URI")
                .unwrap_or_else(|_| "mysql://admin:admin@localhost:3306/testdb".to_string()),
            max_connections: 1,
            sql_type: SqlType::MySql,
        };

        let client = MySqlClient::new(&config).await;

        cleanup(&client.pool).await;

        // Run the SQL script to populate the database
        let script = std::fs::read_to_string("tests/populate_mysql_test.sql").unwrap();
        sqlx::raw_sql(&script).execute(&client.pool).await.unwrap();

        // try name and repository
        let card_args = CardQueryArgs {
            name: Some("Data1".to_string()),
            repository: Some("repo1".to_string()),
            ..Default::default()
        };

        // query all versions
        // get versions (should return 1)
        let results = client
            .query_cards(CardSQLTableNames::Data, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 10);

        // try name and repository
        let card_args = CardQueryArgs {
            name: Some("Model1".to_string()),
            repository: Some("repo1".to_string()),
            version: Some("~1.0.0".to_string()),
            ..Default::default()
        };
        let results = client
            .query_cards(CardSQLTableNames::Model, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        // max_date
        let card_args = CardQueryArgs {
            max_date: Some("2023-11-28".to_string()),
            ..Default::default()
        };
        let results = client
            .query_cards(CardSQLTableNames::Run, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 2);

        // try tags
        let tags = [("key1".to_string(), "value1".to_string())]
            .iter()
            .cloned()
            .collect();
        let card_args = CardQueryArgs {
            tags: Some(tags),
            ..Default::default()
        };
        let results = client
            .query_cards(CardSQLTableNames::Data, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        let card_args = CardQueryArgs {
            sort_by_timestamp: Some(true),
            limit: Some(5),
            ..Default::default()
        };
        let results = client
            .query_cards(CardSQLTableNames::Audit, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 5);

        // test uid
        let card_args = CardQueryArgs {
            uid: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
            ..Default::default()
        };
        let results = client
            .query_cards(CardSQLTableNames::Data, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        cleanup(&client.pool).await;
    }

    #[tokio::test]
    async fn test_mysql_insert_cards() {
        let config = OpsmlDatabaseSettings {
            connection_uri: env::var("OPSML_TRACKING_URI")
                .unwrap_or_else(|_| "mysql://admin:admin@localhost:3306/testdb".to_string()),
            max_connections: 1,
            sql_type: SqlType::MySql,
        };

        let client = MySqlClient::new(&config).await;

        cleanup(&client.pool).await;

        let data_card = DataCardRecord::default();
        let card = Card::Data(data_card.clone());

        client
            .insert_card(CardSQLTableNames::Data, &card)
            .await
            .unwrap();

        // check if the card was inserted
        let card_args = CardQueryArgs {
            uid: Some(data_card.uid),
            ..Default::default()
        };
        let results = client
            .query_cards(CardSQLTableNames::Data, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        // insert modelcard
        let model_card = ModelCardRecord::default();
        let card = Card::Model(model_card.clone());

        client
            .insert_card(CardSQLTableNames::Model, &card)
            .await
            .unwrap();

        // check if the card was inserted
        let card_args = CardQueryArgs {
            uid: Some(model_card.uid),
            ..Default::default()
        };

        let results = client
            .query_cards(CardSQLTableNames::Model, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        // insert runcard
        let run_card = RunCardRecord::default();
        let card = Card::Run(run_card.clone());

        client
            .insert_card(CardSQLTableNames::Run, &card)
            .await
            .unwrap();

        // check if the card was inserted

        let card_args = CardQueryArgs {
            uid: Some(run_card.uid),
            ..Default::default()
        };

        let results = client
            .query_cards(CardSQLTableNames::Run, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        // insert auditcard

        let audit_card = AuditCardRecord::default();
        let card = Card::Audit(audit_card.clone());

        client
            .insert_card(CardSQLTableNames::Audit, &card)
            .await
            .unwrap();

        // check if the card was inserted

        let card_args = CardQueryArgs {
            uid: Some(audit_card.uid),
            ..Default::default()
        };

        let results = client
            .query_cards(CardSQLTableNames::Audit, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        // check pipeline card
        let pipeline_card = PipelineCardRecord::default();
        let card = Card::Pipeline(pipeline_card.clone());

        client
            .insert_card(CardSQLTableNames::Pipeline, &card)
            .await
            .unwrap();

        // check if the card was inserted

        let card_args = CardQueryArgs {
            uid: Some(pipeline_card.uid),
            ..Default::default()
        };

        let results = client
            .query_cards(CardSQLTableNames::Pipeline, &card_args)
            .await
            .unwrap();

        assert_eq!(results.len(), 1);

        cleanup(&client.pool).await;
    }
}

use async_trait::async_trait;
use opsml_error::error::SqlError;
use opsml_settings::config::OpsmlDatabaseSettings;
use std::fmt;

pub enum CardSQLTableNames {
    Data,
    Model,
    Run,
    Project,
    Audit,
    Pipeline,
}

impl fmt::Display for CardSQLTableNames {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let table_name = match self {
            CardSQLTableNames::Data => "opsml_data_registry",
            CardSQLTableNames::Model => "opsml_model_registry",
            CardSQLTableNames::Run => "opsml_run_registry",
            CardSQLTableNames::Project => "opsml_project_registry",
            CardSQLTableNames::Audit => "opsml_audit_registry",
            CardSQLTableNames::Pipeline => "opsml_pipeline_registry",
        };
        write!(f, "{}", table_name)
    }
}

#[async_trait]
pub trait SqlClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self;
    async fn run_migrations(&self) -> Result<(), SqlError>;
    async fn get_versions(
        &self,
        table: CardSQLTableNames,
        name: &str,
        repository: &str,
        version: Option<&str>,
    ) -> Result<Vec<String>, SqlError>;
}

use crate::schemas::schema::VersionResult;
use async_trait::async_trait;
use opsml_error::error::SqlError;
use opsml_settings::config::OpsmlDatabaseSettings;

pub enum CardSQLTableNames {
    Data,
    Model,
    Run,
    Project,
    Audit,
    Pipeline,
}

impl CardSQLTableNames {
    pub fn to_string(&self) -> String {
        match self {
            CardSQLTableNames::Data => "opsml_data_registry".to_string(),
            CardSQLTableNames::Model => "opsml_model_registry".to_string(),
            CardSQLTableNames::Run => "opsml_run_registry".to_string(),
            CardSQLTableNames::Project => "opsml_project_registry".to_string(),
            CardSQLTableNames::Audit => "opsml_audit_registry".to_string(),
            CardSQLTableNames::Pipeline => "opsml_pipeline_registry".to_string(),
        }
    }
}

#[async_trait]
pub trait SqlClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self;
    async fn run_migrations(&self) -> Result<(), SqlError>;
    async fn get_versions(
        &self,
        table: CardSQLTableNames,
        name: Option<&str>,
        repository: Option<&str>,
        version: Option<&str>,
    ) -> Result<Vec<VersionResult>, SqlError>;
}

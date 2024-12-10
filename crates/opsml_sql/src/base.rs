use crate::schemas::arguments::CardQueryArgs;
use crate::schemas::schema::{
    Card, CardResults, CardSummary, HardwareMetricsRecord, MetricRecord, ParameterRecord,
    QueryStats,
};
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
    Metrics,
    HardwareMetrics,
    Parameters,
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
            CardSQLTableNames::Metrics => "opsml_run_metrics",
            CardSQLTableNames::HardwareMetrics => "opsml_run_hardware_metrics",
            CardSQLTableNames::Parameters => "opsml_run_parameters",
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

    async fn query_cards(
        &self,
        table: CardSQLTableNames,
        query_args: &CardQueryArgs,
    ) -> Result<CardResults, SqlError>;

    async fn insert_card(&self, table: CardSQLTableNames, card: &Card) -> Result<(), SqlError>;
    async fn update_card(&self, table: CardSQLTableNames, card: &Card) -> Result<(), SqlError>;
    async fn get_unique_repository_names(
        &self,
        table: CardSQLTableNames,
    ) -> Result<Vec<String>, SqlError>;
    async fn query_stats(
        &self,
        table: CardSQLTableNames,
        search_term: Option<&str>,
    ) -> Result<QueryStats, SqlError>;

    async fn query_page(
        &self,
        sort_by: &str,
        page: i64,
        search_term: Option<&str>,
        repository: Option<&str>,
        table: CardSQLTableNames,
    ) -> Result<Vec<CardSummary>, SqlError>;

    async fn delete_card(&self, table: CardSQLTableNames, uid: &str) -> Result<(), SqlError>;

    // db specific functions
    // get project_id
    async fn get_project_id(&self, project_name: &str, repository: &str) -> Result<i32, SqlError>;

    /// Insert run metric
    ///
    /// # Arguments
    ///
    /// * `card` - The metric record
    ///
    /// # Returns
    ///
    async fn insert_run_metric(&self, card: &MetricRecord) -> Result<(), SqlError>;

    /// insert run parameter
    ///
    /// # Arguments
    ///
    /// * `card` - The parameter record
    ///
    /// # Returns
    ///
    async fn insert_run_parameter(&self, card: &ParameterRecord) -> Result<(), SqlError>;

    /// Get run metric
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the card
    /// * `names` - The names of the metrics
    ///
    /// # Returns
    ///
    /// * `Vec<MetricRecord>` - The metrics
    ///
    async fn get_run_metric(
        &self,
        uid: &str,
        names: Option<&Vec<&str>>,
    ) -> Result<Vec<MetricRecord>, SqlError>;

    /// Get run metric names
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the card
    ///
    /// # Returns
    ///
    /// * `Vec<String>` - The names of the metrics
    ///
    async fn get_run_metric_names(&self, uid: &str) -> Result<Vec<String>, SqlError>;

    /// Get run parameter
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the card
    /// * `names` - The names of the parameters
    ///
    /// # Returns
    ///
    /// * `Vec<ParameterRecord>` - The parameters
    async fn get_run_parameter(
        &self,
        uid: &str,
        names: Option<&Vec<&str>>,
    ) -> Result<Vec<ParameterRecord>, SqlError>;

    /// Insert hardware metrics
    ///
    /// # Arguments
    ///
    /// * `metric_record` - The hardware metrics
    ///
    async fn insert_hardware_metric(
        &self,
        metric_record: &HardwareMetricsRecord,
    ) -> Result<(), SqlError>;

    /// Get hardware metrics
    ///
    /// # Arguments
    ///
    /// * `uid` - The unique identifier of the card
    ///
    /// # Returns
    ///
    /// * `HardwareMetricsRecord` - The hardware metrics

    async fn get_hardware_metric(&self, uid: &str) -> Result<Vec<HardwareMetricsRecord>, SqlError>;
}

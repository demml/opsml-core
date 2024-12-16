use crate::schemas::arguments::CardQueryArgs;
use crate::schemas::schema::{
    Card, CardResults, CardSummary, HardwareMetricsRecord, MetricRecord, ParameterRecord,
    QueryStats, User,
};
use async_trait::async_trait;
use opsml_error::error::SqlError;
use opsml_settings::config::OpsmlDatabaseSettings;
use opsml_utils::semver::VersionParser;
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
    Users,
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
            CardSQLTableNames::Users => "opsml_users",
        };
        write!(f, "{}", table_name)
    }
}

pub fn add_version_bounds(builder: &mut String, version: &str) -> Result<(), SqlError> {
    let version_bounds = VersionParser::get_version_to_search(version)
        .map_err(|e| SqlError::VersionError(format!("{}", e)))?;

    // construct lower bound (already validated)
    builder.push_str(
        format!(
            " AND (major >= {} AND minor >= {} and patch >= {})",
            version_bounds.lower_bound.major,
            version_bounds.lower_bound.minor,
            version_bounds.lower_bound.patch
        )
        .as_str(),
    );

    if !version_bounds.no_upper_bound {
        // construct upper bound based on number of components
        if version_bounds.num_parts == 1 {
            builder
                .push_str(format!(" AND (major < {})", version_bounds.upper_bound.major).as_str());
        } else if version_bounds.num_parts == 2
            || version_bounds.num_parts == 3 && version_bounds.parser_type == VersionParser::Tilde
            || version_bounds.num_parts == 3 && version_bounds.parser_type == VersionParser::Caret
        {
            builder.push_str(
                format!(
                    " AND (major = {} AND minor < {})",
                    version_bounds.upper_bound.major, version_bounds.upper_bound.minor
                )
                .as_str(),
            );
        } else {
            builder.push_str(
                format!(
                    " AND (major = {} AND minor = {} AND patch < {})",
                    version_bounds.upper_bound.major,
                    version_bounds.upper_bound.minor,
                    version_bounds.upper_bound.patch
                )
                .as_str(),
            );
        }
    }
    Ok(())
}

#[async_trait]
pub trait SqlClient: Sized {
    async fn new(settings: &OpsmlDatabaseSettings) -> Result<Self, SqlError>;
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
    async fn insert_run_metric(&self, record: &MetricRecord) -> Result<(), SqlError>;

    /// insert run parameter
    ///
    /// # Arguments
    ///
    /// * `card` - The parameter record
    ///
    /// # Returns
    ///
    async fn insert_run_parameter(&self, record: &ParameterRecord) -> Result<(), SqlError>;

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
    async fn insert_hardware_metric(&self, record: &HardwareMetricsRecord) -> Result<(), SqlError>;

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

    /// Insert user
    ///
    /// # Arguments
    ///
    /// * `user` - The user
    ///
    async fn insert_user(&self, user: &User) -> Result<(), SqlError>;

    /// Get user
    ///
    /// # Arguments
    ///
    /// * `username` - The username
    ///
    /// # Returns
    ///
    /// * `User` - The user
    async fn get_user(&self, username: &str) -> Result<User, SqlError>;

    /// update user
    ///
    /// # Arguments
    ///
    /// * `user` - The user
    ///
    /// # Returns
    ///
    /// * `Result<(), SqlError>` - The result of the operation
    async fn update_user(&self, user: &User) -> Result<(), SqlError>;
}

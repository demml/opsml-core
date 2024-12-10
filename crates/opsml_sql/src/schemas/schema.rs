use chrono::NaiveDateTime;
use chrono::Utc;
use opsml_error::error::VersionError;
use opsml_types::enums::CommonKwargs;
use opsml_types::types::HardwareMetrics;
use opsml_utils::utils::{get_utc_date, get_utc_timestamp};
use semver::Op;
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MetricRecord {
    pub run_uid: String,
    pub name: String,
    pub value: f64,
    pub step: Option<i32>,
    pub timestamp: Option<i64>,
    pub date_ts: String,
    pub idx: Option<i32>,
}

impl MetricRecord {
    pub fn new(
        run_uid: String,
        name: String,
        value: f64,
        step: Option<i32>,
        timestamp: Option<i64>,
    ) -> Self {
        MetricRecord {
            run_uid,
            name,
            value,
            step,
            timestamp,
            date_ts: get_utc_date(),
            idx: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VersionResult {
    pub date: String,
    pub timestamp: i64,
    pub name: String,
    pub repository: String,
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub pre_tag: Option<String>,
    pub build_tag: Option<String>,
}

impl VersionResult {
    pub fn to_version(&self) -> Result<Version, VersionError> {
        let mut version = Version::new(self.major as u64, self.minor as u64, self.patch as u64);

        if self.pre_tag.is_some() {
            version.pre = Prerelease::new(self.pre_tag.as_ref().unwrap())
                .map_err(|e| VersionError::InvalidPreRelease(format!("{}", e)))?;
        }

        if self.build_tag.is_some() {
            version.build = BuildMetadata::new(self.build_tag.as_ref().unwrap())
                .map_err(|e| VersionError::InvalidBuild(format!("{}", e)))?;
        }

        Ok(version)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Repository {
    pub repository: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QueryStats {
    pub nbr_names: i32,
    pub nbr_repositories: i32,
    pub nbr_versions: i32,
}

#[derive(Debug, FromRow)]
pub struct CardSummary {
    pub repository: String,
    pub name: String,
    pub version: String,
    pub versions: i64,
    pub updated_at: i64,
    pub created_at: i64,
    pub row_num: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataCardRecord {
    pub uid: String,
    pub date: String,
    pub timestamp: i64,
    pub app_env: String,
    pub name: String,
    pub repository: String,
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub pre_tag: Option<String>,
    pub build_tag: Option<String>,
    pub version: String,
    pub contact: String,
    pub tags: Json<HashMap<String, String>>,
    pub data_type: String,
    pub runcard_uid: String,
    pub pipelinecard_uid: String,
    pub auditcard_uid: String,
    pub interface_type: String,
}

#[allow(clippy::too_many_arguments)]
impl DataCardRecord {
    pub fn new(
        name: String,
        repository: String,
        version: Version,
        contact: String,
        tags: Option<HashMap<String, String>>,
        data_type: String,
        runcard_uid: Option<String>,
        pipelinecard_uid: Option<String>,
        auditcard_uid: Option<String>,
        interface_type: Option<String>,
    ) -> Self {
        let date = get_utc_date();
        let timestamp = get_utc_timestamp();
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let uid = Uuid::new_v4().to_string();

        DataCardRecord {
            uid,
            date,
            timestamp,
            app_env,
            name,
            repository,
            major: version.major as i32,
            minor: version.minor as i32,
            patch: version.patch as i32,
            pre_tag: version.pre.to_string().parse().ok(),
            build_tag: version.build.to_string().parse().ok(),
            version: version.to_string(),
            contact,
            tags: Json(tags.unwrap_or_default()),
            data_type,
            runcard_uid: runcard_uid
                .unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
            pipelinecard_uid: pipelinecard_uid
                .unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
            auditcard_uid: auditcard_uid
                .unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
            interface_type: interface_type
                .unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
        }
    }
}

impl Default for DataCardRecord {
    fn default() -> Self {
        DataCardRecord {
            uid: Uuid::new_v4().to_string(),
            date: get_utc_date(),
            timestamp: get_utc_timestamp(),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            name: CommonKwargs::Undefined.as_string().to_string(),
            repository: CommonKwargs::Undefined.as_string().to_string(),
            major: 1,
            minor: 0,
            patch: 0,
            pre_tag: None,
            build_tag: None,
            version: Version::new(1, 0, 0).to_string(),
            contact: CommonKwargs::Undefined.as_string().to_string(),
            tags: Json(HashMap::new()),
            data_type: CommonKwargs::Undefined.as_string().to_string(),
            runcard_uid: CommonKwargs::Undefined.as_string().to_string(),
            pipelinecard_uid: CommonKwargs::Undefined.as_string().to_string(),
            auditcard_uid: CommonKwargs::Undefined.as_string().to_string(),
            interface_type: CommonKwargs::Undefined.as_string().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelCardRecord {
    pub uid: String,
    pub date: String,
    pub timestamp: i64,
    pub app_env: String,
    pub name: String,
    pub repository: String,
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub pre_tag: Option<String>,
    pub build_tag: Option<String>,
    pub version: String,
    pub contact: String,
    pub tags: Json<HashMap<String, String>>,
    pub datacard_uid: String,
    pub sample_data_type: String,
    pub model_type: String,
    pub runcard_uid: String,
    pub pipelinecard_uid: String,
    pub auditcard_uid: String,
    pub interface_type: String,
    pub task_type: String,
}

#[allow(clippy::too_many_arguments)]
impl ModelCardRecord {
    pub fn new(
        name: String,
        repository: String,
        version: Version,
        contact: String,
        tags: Option<HashMap<String, String>>,
        datacard_uid: Option<String>,
        sample_data_type: String,
        model_type: String,
        runcard_uid: Option<String>,
        pipelinecard_uid: Option<String>,
        auditcard_uid: Option<String>,
        interface_type: Option<String>,
        task_type: Option<String>,
    ) -> Self {
        let date = get_utc_date();
        let timestamp = get_utc_timestamp();
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let uid = Uuid::new_v4().to_string();

        ModelCardRecord {
            uid,
            date,
            timestamp,
            app_env,
            name,
            repository,
            major: version.major as i32,
            minor: version.minor as i32,
            patch: version.patch as i32,
            pre_tag: version.pre.to_string().parse().ok(),
            build_tag: version.build.to_string().parse().ok(),
            version: version.to_string(),
            contact,
            tags: Json(tags.unwrap_or_default()),
            datacard_uid: datacard_uid
                .unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
            sample_data_type,
            model_type,
            runcard_uid: runcard_uid
                .unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
            pipelinecard_uid: pipelinecard_uid
                .unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
            auditcard_uid: auditcard_uid
                .unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
            interface_type: interface_type
                .unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
            task_type: task_type.unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
        }
    }
}

impl Default for ModelCardRecord {
    fn default() -> Self {
        ModelCardRecord {
            uid: Uuid::new_v4().to_string(),
            date: get_utc_date(),
            timestamp: get_utc_timestamp(),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            name: CommonKwargs::Undefined.as_string().to_string(),
            repository: CommonKwargs::Undefined.as_string().to_string(),
            major: 1,
            minor: 0,
            patch: 0,
            pre_tag: None,
            build_tag: None,
            version: Version::new(1, 0, 0).to_string(),
            contact: CommonKwargs::Undefined.as_string().to_string(),
            tags: Json(HashMap::new()),
            datacard_uid: CommonKwargs::Undefined.as_string().to_string(),
            sample_data_type: CommonKwargs::Undefined.as_string().to_string(),
            model_type: CommonKwargs::Undefined.as_string().to_string(),
            runcard_uid: CommonKwargs::Undefined.as_string().to_string(),
            pipelinecard_uid: CommonKwargs::Undefined.as_string().to_string(),
            auditcard_uid: CommonKwargs::Undefined.as_string().to_string(),
            interface_type: CommonKwargs::Undefined.as_string().to_string(),
            task_type: CommonKwargs::Undefined.as_string().to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RunCardRecord {
    pub uid: String,
    pub date: String,
    pub timestamp: i64,
    pub app_env: String,
    pub name: String,
    pub repository: String,
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub pre_tag: Option<String>,
    pub build_tag: Option<String>,
    pub version: String,
    pub contact: String,
    pub tags: Json<HashMap<String, String>>,
    pub datacard_uids: Json<Vec<String>>,
    pub modelcard_uids: Json<Vec<String>>,
    pub pipelinecard_uid: String,
    pub project: String,
    pub artifact_uris: Json<HashMap<String, String>>,
    pub compute_environment: Json<HashMap<String, String>>,
}

impl Default for RunCardRecord {
    fn default() -> Self {
        RunCardRecord {
            uid: Uuid::new_v4().to_string(),
            date: get_utc_date(),
            timestamp: get_utc_timestamp(),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            name: CommonKwargs::Undefined.as_string().to_string(),
            repository: CommonKwargs::Undefined.as_string().to_string(),
            major: 1,
            minor: 0,
            patch: 0,
            pre_tag: None,
            build_tag: None,
            version: Version::new(1, 0, 0).to_string(),
            contact: CommonKwargs::Undefined.as_string().to_string(),
            tags: Json(HashMap::new()),
            datacard_uids: Json(Vec::new()),
            modelcard_uids: Json(Vec::new()),
            pipelinecard_uid: CommonKwargs::Undefined.as_string().to_string(),
            project: CommonKwargs::Undefined.as_string().to_string(),
            artifact_uris: Json(HashMap::new()),
            compute_environment: Json(HashMap::new()),
        }
    }
}

#[allow(clippy::too_many_arguments)]
impl RunCardRecord {
    pub fn new(
        name: String,
        repository: String,
        version: Version,
        contact: String,
        tags: Option<HashMap<String, String>>,
        datacard_uids: Option<Vec<String>>,
        modelcard_uids: Option<Vec<String>>,
        pipelinecard_uid: Option<String>,
        project: String,
        artifact_uris: Option<HashMap<String, String>>,
        compute_environment: Option<HashMap<String, String>>,
    ) -> Self {
        let date = get_utc_date();
        let timestamp = get_utc_timestamp();
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let uid = Uuid::new_v4().to_string();

        RunCardRecord {
            uid,
            date,
            timestamp,
            app_env,
            name,
            repository,
            major: version.major as i32,
            minor: version.minor as i32,
            patch: version.patch as i32,
            pre_tag: version.pre.to_string().parse().ok(),
            build_tag: version.build.to_string().parse().ok(),
            version: version.to_string(),
            contact,
            tags: Json(tags.unwrap_or_default()),
            datacard_uids: Json(datacard_uids.unwrap_or_default()),
            modelcard_uids: Json(modelcard_uids.unwrap_or_default()),
            pipelinecard_uid: pipelinecard_uid
                .unwrap_or_else(|| CommonKwargs::Undefined.as_string().to_string()),
            project,
            artifact_uris: Json(artifact_uris.unwrap_or_default()),
            compute_environment: Json(compute_environment.unwrap_or_default()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditCardRecord {
    pub uid: String,
    pub date: String,
    pub timestamp: i64,
    pub app_env: String,
    pub name: String,
    pub repository: String,
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub pre_tag: Option<String>,
    pub build_tag: Option<String>,
    pub version: String,
    pub contact: String,
    pub tags: Json<HashMap<String, String>>,
    pub approved: bool,
    pub datacard_uids: Json<Vec<String>>,
    pub modelcard_uids: Json<Vec<String>>,
    pub runcard_uids: Json<Vec<String>>,
}

#[allow(clippy::too_many_arguments)]
impl AuditCardRecord {
    pub fn new(
        name: String,
        repository: String,
        version: Version,
        contact: String,
        tags: Option<HashMap<String, String>>,
        approved: bool,
        datacard_uids: Option<Vec<String>>,
        modelcard_uids: Option<Vec<String>>,
        runcard_uids: Option<Vec<String>>,
    ) -> Self {
        let date = get_utc_date();
        let timestamp = get_utc_timestamp();
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let uid = Uuid::new_v4().to_string();

        AuditCardRecord {
            uid,
            date,
            timestamp,
            app_env,
            name,
            repository,
            major: version.major as i32,
            minor: version.minor as i32,
            patch: version.patch as i32,
            pre_tag: version.pre.to_string().parse().ok(),
            build_tag: version.build.to_string().parse().ok(),
            version: version.to_string(),
            contact,
            tags: Json(tags.unwrap_or_default()),
            approved,
            datacard_uids: Json(datacard_uids.unwrap_or_default()),
            modelcard_uids: Json(modelcard_uids.unwrap_or_default()),
            runcard_uids: Json(runcard_uids.unwrap_or_default()),
        }
    }
}

impl Default for AuditCardRecord {
    fn default() -> Self {
        AuditCardRecord {
            uid: Uuid::new_v4().to_string(),
            date: get_utc_date(),
            timestamp: get_utc_timestamp(),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            name: CommonKwargs::Undefined.as_string().to_string(),
            repository: CommonKwargs::Undefined.as_string().to_string(),
            major: 1,
            minor: 0,
            patch: 0,
            pre_tag: None,
            build_tag: None,
            version: Version::new(1, 0, 0).to_string(),
            contact: CommonKwargs::Undefined.as_string().to_string(),
            tags: Json(HashMap::new()),
            approved: false,
            datacard_uids: Json(Vec::new()),
            modelcard_uids: Json(Vec::new()),
            runcard_uids: Json(Vec::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipelineCardRecord {
    pub uid: String,
    pub date: String,
    pub timestamp: i64,
    pub app_env: String,
    pub name: String,
    pub repository: String,
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub pre_tag: Option<String>,
    pub build_tag: Option<String>,
    pub version: String,
    pub contact: String,
    pub tags: Json<HashMap<String, String>>,
    pub pipeline_code_uri: String,
    pub datacard_uids: Json<Vec<String>>,
    pub modelcard_uids: Json<Vec<String>>,
    pub runcard_uids: Json<Vec<String>>,
}

#[allow(clippy::too_many_arguments)]
impl PipelineCardRecord {
    pub fn new(
        name: String,
        repository: String,
        version: Version,
        contact: String,
        tags: Option<HashMap<String, String>>,
        pipeline_code_uri: String,
        datacard_uids: Option<Vec<String>>,
        modelcard_uids: Option<Vec<String>>,
        runcard_uids: Option<Vec<String>>,
    ) -> Self {
        let date = get_utc_date();
        let timestamp = get_utc_timestamp();
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let uid = Uuid::new_v4().to_string();

        PipelineCardRecord {
            uid,
            date,
            timestamp,
            app_env,
            name,
            repository,
            major: version.major as i32,
            minor: version.minor as i32,
            patch: version.patch as i32,
            pre_tag: version.pre.to_string().parse().ok(),
            build_tag: version.build.to_string().parse().ok(),
            version: version.to_string(),
            contact,
            tags: Json(tags.unwrap_or_default()),
            pipeline_code_uri,
            datacard_uids: Json(datacard_uids.unwrap_or_default()),
            modelcard_uids: Json(modelcard_uids.unwrap_or_default()),
            runcard_uids: Json(runcard_uids.unwrap_or_default()),
        }
    }
}

impl Default for PipelineCardRecord {
    fn default() -> Self {
        PipelineCardRecord {
            uid: Uuid::new_v4().to_string(),
            date: get_utc_date(),
            timestamp: get_utc_timestamp(),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            name: CommonKwargs::Undefined.as_string().to_string(),
            repository: CommonKwargs::Undefined.as_string().to_string(),
            major: 1,
            minor: 0,
            patch: 0,
            pre_tag: None,
            build_tag: None,
            version: Version::new(1, 0, 0).to_string(),
            contact: CommonKwargs::Undefined.as_string().to_string(),
            tags: Json(HashMap::new()),
            pipeline_code_uri: CommonKwargs::Undefined.as_string().to_string(),
            datacard_uids: Json(Vec::new()),
            modelcard_uids: Json(Vec::new()),
            runcard_uids: Json(Vec::new()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProjectCardRecord {
    pub date: String,
    pub uid: String,
    pub name: String,
    pub repository: String,
    pub project_id: i32,
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
    pub pre_tag: Option<String>,
    pub build_tag: Option<String>,
    pub version: String,
    pub timestamp: i64,
}

impl Default for ProjectCardRecord {
    fn default() -> Self {
        ProjectCardRecord {
            date: get_utc_date(),
            uid: Uuid::new_v4().to_string(),
            name: CommonKwargs::Undefined.as_string().to_string(),
            repository: CommonKwargs::Undefined.as_string().to_string(),
            project_id: 1,
            major: 1,
            minor: 0,
            patch: 0,
            pre_tag: None,
            build_tag: None,
            version: Version::new(1, 0, 0).to_string(),
            timestamp: get_utc_timestamp(),
        }
    }
}

impl ProjectCardRecord {
    pub fn new(name: String, repository: String, version: Version, project_id: i32) -> Self {
        ProjectCardRecord {
            date: get_utc_date(),
            uid: Uuid::new_v4().to_string(),
            name,
            repository,
            project_id,
            major: version.major as i32,
            minor: version.minor as i32,
            patch: version.patch as i32,
            pre_tag: version.pre.to_string().parse().ok(),
            build_tag: version.build.to_string().parse().ok(),
            version: version.to_string(),
            timestamp: get_utc_timestamp(),
        }
    }
}

// create enum that takes vec of cards

#[derive(Debug)]
pub enum CardResults {
    Data(Vec<DataCardRecord>),
    Model(Vec<ModelCardRecord>),
    Run(Vec<RunCardRecord>),
    Audit(Vec<AuditCardRecord>),
    Pipeline(Vec<PipelineCardRecord>),
    Project(Vec<ProjectCardRecord>),
}

impl CardResults {
    pub fn len(&self) -> usize {
        match self {
            CardResults::Data(cards) => cards.len(),
            CardResults::Model(cards) => cards.len(),
            CardResults::Run(cards) => cards.len(),
            CardResults::Audit(cards) => cards.len(),
            CardResults::Pipeline(cards) => cards.len(),
            CardResults::Project(cards) => cards.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            CardResults::Data(cards) => cards.is_empty(),
            CardResults::Model(cards) => cards.is_empty(),
            CardResults::Run(cards) => cards.is_empty(),
            CardResults::Audit(cards) => cards.is_empty(),
            CardResults::Pipeline(cards) => cards.is_empty(),
            CardResults::Project(cards) => cards.is_empty(),
        }
    }
    pub fn to_json(&self) -> Vec<String> {
        match self {
            CardResults::Data(cards) => cards
                .iter()
                .map(|card| serde_json::to_string_pretty(card).unwrap())
                .collect(),
            CardResults::Model(cards) => cards
                .iter()
                .map(|card| serde_json::to_string_pretty(card).unwrap())
                .collect(),
            CardResults::Run(cards) => cards
                .iter()
                .map(|card| serde_json::to_string_pretty(card).unwrap())
                .collect(),
            CardResults::Audit(cards) => cards
                .iter()
                .map(|card| serde_json::to_string_pretty(card).unwrap())
                .collect(),
            CardResults::Pipeline(cards) => cards
                .iter()
                .map(|card| serde_json::to_string_pretty(card).unwrap())
                .collect(),
            CardResults::Project(cards) => cards
                .iter()
                .map(|card| serde_json::to_string_pretty(card).unwrap())
                .collect(),
        }
    }
}

#[derive(Debug)]
pub enum Card {
    Data(DataCardRecord),
    Model(ModelCardRecord),
    Run(RunCardRecord),
    Audit(AuditCardRecord),
    Pipeline(PipelineCardRecord),
    Project(ProjectCardRecord),
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HardwareMetricsRecord {
    pub run_uid: String,
    pub created_at: NaiveDateTime,
    pub cpu_percent_utilization: f64,
    pub cpu_percent_per_core: Option<Json<Vec<f64>>>,
    pub compute_overall: Option<f64>,
    pub compute_utilized: Option<f64>,
    pub load_avg: f64,
    pub sys_ram_total: i64,
    pub sys_ram_used: i64,
    pub sys_ram_available: i64,
    pub sys_ram_percent_used: f64,
    pub sys_swap_total: Option<i64>,
    pub sys_swap_used: Option<i64>,
    pub sys_swap_free: Option<i64>,
    pub sys_swap_percent: Option<f64>,
    pub bytes_recv: i64,
    pub bytes_sent: i64,
    pub gpu_percent_utilization: Option<f64>,
    pub gpu_percent_per_core: Option<Json<Vec<f64>>>,
}

impl Default for HardwareMetricsRecord {
    fn default() -> Self {
        HardwareMetricsRecord {
            run_uid: Uuid::new_v4().to_string(),
            created_at: Utc::now().naive_utc(),
            cpu_percent_utilization: 0.0,
            cpu_percent_per_core: None,
            compute_overall: None,
            compute_utilized: None,
            load_avg: 0.0,
            sys_ram_total: 0,
            sys_ram_used: 0,
            sys_ram_available: 0,
            sys_ram_percent_used: 0.0,
            sys_swap_total: None,
            sys_swap_used: None,
            sys_swap_free: None,
            sys_swap_percent: None,
            bytes_recv: 0,
            bytes_sent: 0,
            gpu_percent_utilization: None,
            gpu_percent_per_core: None,
        }
    }
}

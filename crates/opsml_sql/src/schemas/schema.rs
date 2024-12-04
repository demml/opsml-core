use opsml_error::error::VersionError;
use semver::{BuildMetadata, Prerelease, Version};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use std::collections::HashMap;

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
pub struct DataCardResult {
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
    pub contact: String,
    pub tags: Json<HashMap<String, String>>,
    pub data_type: String,
    pub runcard_uid: String,
    pub pipelinecard_uid: String,
    pub auditcard_uid: String,
    pub interface_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelCardResult {
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RunCardResult {
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
    pub contact: String,
    pub tags: Json<HashMap<String, String>>,
    pub datacard_uids: Json<HashMap<String, String>>,
    pub modelcard_uids: Json<HashMap<String, String>>,
    pub pipelinecard_uid: String,
    pub project: String,
    pub artifact_uris: Json<HashMap<String, String>>,
    pub compute_environment: Json<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditCardResult {
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
    pub contact: String,
    pub tags: Json<HashMap<String, String>>,
    pub approved: bool,
    pub datacards: Json<HashMap<String, String>>,
    pub modelcards: Json<HashMap<String, String>>,
    pub runcards: Json<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PipelineCardResult {
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
    pub contact: String,
    pub tags: Json<HashMap<String, String>>,
    pub pipeline_code_uri: String,
    pub datacard_uids: Json<HashMap<String, String>>,
    pub modelcard_uids: Json<HashMap<String, String>>,
    pub runcard_uids: Json<HashMap<String, String>>,
}

// create enum that takes vec of cards
pub enum CardResults {
    Data(Vec<DataCardResult>),
    Model(Vec<ModelCardResult>),
    Run(Vec<RunCardResult>),
    Audit(Vec<AuditCardResult>),
    Pipeline(Vec<PipelineCardResult>),
}

use crate::helper::PyHelperFuncs;
use crate::VersionType;
use crate::{enums::StorageType, RegistryType};
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[pyclass]
pub struct FileInfo {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub size: i64,
    #[pyo3(get)]
    pub object_type: String,
    #[pyo3(get)]
    pub created: String,
    #[pyo3(get)]
    pub suffix: String,
}

#[pymethods]
impl FileInfo {
    pub fn __str__(&self) -> String {
        // serialize the struct to a string
        PyHelperFuncs::__str__(self)
    }
}

#[derive(Serialize, Deserialize)]
pub struct PresignedUrl {
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListFileResponse {
    pub files: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ListFileInfoResponse {
    pub files: Vec<FileInfo>,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteFileResponse {
    pub deleted: bool,
}

#[derive(Serialize, Deserialize)]
pub struct MultiPartSession {
    pub session_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageSettings {
    pub storage_type: StorageType,
}

#[derive(Serialize, Deserialize)]
pub struct UploadResponse {
    pub uploaded: bool,
}

pub struct UploadPartArgs {
    pub presigned_url: Option<String>,
    pub chunk_size: u64,
    pub chunk_index: u64,
    pub this_chunk_size: u64,
}

#[derive(Serialize, Deserialize)]
pub struct UidRequest {
    pub uid: String,
    pub registry_type: RegistryType,
}

#[derive(Serialize, Deserialize)]
pub struct UidResponse {
    pub exists: bool,
}

#[derive(Serialize, Deserialize)]
pub struct RepositoryRequest {
    pub registry_type: RegistryType,
}

#[derive(Serialize, Deserialize)]
pub struct RepositoryResponse {
    pub repositories: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RegistryStatsRequest {
    pub registry_type: RegistryType,
    pub search_term: Option<String>,
}

// RegistryStatsResponse is sourced from sql schema

#[derive(Serialize, Deserialize)]
pub struct QueryPageRequest {
    pub registry_type: RegistryType,
    pub sort_by: Option<String>,
    pub repository: Option<String>,
    pub search_term: Option<String>,
    pub page: Option<i32>,
}

// QueryPageResponse is sourced from sql schema

#[derive(Serialize, Deserialize)]
pub struct CardVersionRequest {
    pub registry_type: RegistryType,
    pub name: String,
    pub repository: String,
    pub version: Option<String>,
    pub version_type: VersionType,
    pub pre_tag: Option<String>,
    pub build_tag: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CardVersionResponse {
    pub version: String,
}

/// Arguments for querying cards
///
/// # Fields
///
/// * `uid` - The unique identifier of the card
/// * `name` - The name of the card
/// * `repository` - The repository of the card
/// * `version` - The version of the card
/// * `max_date` - The maximum date of the card
/// * `tags` - The tags of the card
/// * `limit` - The maximum number of cards to return
/// * `query_terms` - The query terms to search for
/// * `sort_by_timestamp` - Whether to sort by timestamp

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CardQueryArgs {
    pub uid: Option<String>,
    pub name: Option<String>,
    pub repository: Option<String>,
    pub version: Option<String>,
    pub max_date: Option<String>,
    pub tags: Option<HashMap<String, String>>,
    pub limit: Option<i32>,
    pub sort_by_timestamp: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListCardRequest {
    pub uid: Option<String>,
    pub name: Option<String>,
    pub repository: Option<String>,
    pub version: Option<String>,
    pub max_date: Option<String>,
    pub tags: Option<HashMap<String, String>>,
    pub limit: Option<i32>,
    pub sort_by_timestamp: Option<bool>,
    pub registry_type: RegistryType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCardClientRecord {
    pub name: String,
    pub repository: String,
    pub version: String,
    pub contact: String,
    pub tags: HashMap<String, String>,
    pub data_type: String,
    pub runcard_uid: Option<String>,
    pub pipelinecard_uid: Option<String>,
    pub auditcard_uid: Option<String>,
    pub interface_type: Option<String>,
}

impl Default for DataCardClientRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            repository: "".to_string(),
            version: "".to_string(),
            contact: "".to_string(),
            tags: HashMap::new(),
            data_type: "".to_string(),
            runcard_uid: None,
            pipelinecard_uid: None,
            auditcard_uid: None,
            interface_type: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCardClientRecord {
    pub name: String,
    pub repository: String,
    pub version: String,
    pub contact: String,
    pub tags: HashMap<String, String>,
    pub datacard_uid: Option<String>,
    pub sample_data_type: String,
    pub model_type: String,
    pub runcard_uid: Option<String>,
    pub pipelinecard_uid: Option<String>,
    pub auditcard_uid: Option<String>,
    pub interface_type: Option<String>,
    pub task_type: Option<String>,
}

impl Default for ModelCardClientRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            repository: "".to_string(),
            version: "".to_string(),
            contact: "".to_string(),
            tags: HashMap::new(),
            datacard_uid: None,
            sample_data_type: "".to_string(),
            model_type: "".to_string(),
            runcard_uid: None,
            pipelinecard_uid: None,
            auditcard_uid: None,
            interface_type: None,
            task_type: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunCardClientRecord {
    pub name: String,
    pub repository: String,
    pub version: String,
    pub contact: String,
    pub tags: HashMap<String, String>,
    pub datacard_uids: Option<Vec<String>>,
    pub modelcard_uids: Option<Vec<String>>,
    pub pipelinecard_uid: Option<String>,
    pub project: String,
    pub artifact_uris: Option<HashMap<String, String>>,
    pub compute_environment: Option<HashMap<String, String>>,
}

impl Default for RunCardClientRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            repository: "".to_string(),
            version: "".to_string(),
            contact: "".to_string(),
            tags: HashMap::new(),
            datacard_uids: None,
            modelcard_uids: None,
            pipelinecard_uid: None,
            project: "".to_string(),
            artifact_uris: None,
            compute_environment: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditCardClientRecord {
    pub name: String,
    pub repository: String,
    pub version: String,
    pub contact: String,
    pub tags: HashMap<String, String>,
    pub approved: bool,
    pub datacard_uids: Option<Vec<String>>,
    pub modelcard_uids: Option<Vec<String>>,
    pub runcard_uids: Option<Vec<String>>,
}

impl Default for AuditCardClientRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            repository: "".to_string(),
            version: "".to_string(),
            contact: "".to_string(),
            tags: HashMap::new(),
            approved: false,
            datacard_uids: None,
            modelcard_uids: None,
            runcard_uids: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineCardClientRecord {
    pub name: String,
    pub repository: String,
    pub version: String,
    pub contact: String,
    pub tags: HashMap<String, String>,
    pub pipeline_code_uri: String,
    pub datacard_uids: Option<Vec<String>>,
    pub modelcard_uids: Option<Vec<String>>,
    pub runcard_uids: Option<Vec<String>>,
}

impl Default for PipelineCardClientRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            repository: "".to_string(),
            version: "".to_string(),
            contact: "".to_string(),
            tags: HashMap::new(),
            pipeline_code_uri: "".to_string(),
            datacard_uids: None,
            modelcard_uids: None,
            runcard_uids: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCardClientRecord {
    pub name: String,
    pub repository: String,
    pub version: String,
    pub project_id: i32,
}

impl Default for ProjectCardClientRecord {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            repository: "".to_string(),
            version: "".to_string(),
            project_id: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientCard {
    Data(DataCardClientRecord),
    Model(ModelCardClientRecord),
    Run(RunCardClientRecord),
    Audit(AuditCardClientRecord),
    Pipeline(PipelineCardClientRecord),
    Project(ProjectCardClientRecord),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCardRequest {
    pub registry_type: RegistryType,
    pub card: ClientCard,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCardResponse {
    pub registered: bool,
}

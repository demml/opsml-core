use axum::http::HeaderMap;
use opsml_storage::storage::base::UploadPartArgs;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct MultiPartQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct PresignedQuery {
    pub path: String,
    pub session_url: Option<String>,
    pub part_number: Option<i32>,
    pub for_multi_part: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct UploadPartArgParser {}

impl UploadPartArgParser {
    pub fn to_args(headers: HeaderMap) -> UploadPartArgs {
        let path = headers
            .get("File-Path")
            .and_then(|v| v.to_str().ok())
            .map(Path::new)
            .unwrap_or(Path::new(""))
            .to_path_buf();

        UploadPartArgs { path }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ListFileQuery {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct DeleteFileQuery {
    pub path: String,
    pub recursive: bool,
}

#[derive(Serialize, Deserialize)]
pub struct DownloadFileQuery {
    pub path: String,
}

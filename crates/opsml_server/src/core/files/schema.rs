use axum::Json;
use axum::{http::HeaderMap, response::IntoResponse};
use opsml_storage::core::storage::base::UploadPartArgs;
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
pub struct MultiPartSession {
    pub session_url: String,
}

// Implement IntoResponse for Alive
impl IntoResponse for MultiPartSession {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PresignedUrl {
    pub url: String,
}

// Implement IntoResponse for Alive
impl IntoResponse for PresignedUrl {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
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

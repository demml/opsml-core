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
pub struct ResumableSession {
    pub session_uri: String,
}

// Implement IntoResponse for Alive
impl IntoResponse for ResumableSession {
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
            .map(|v| Path::new(v))
            .unwrap_or(Path::new(""))
            .to_path_buf();

        UploadPartArgs { path }
    }
}

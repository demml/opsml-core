use axum::Json;
use axum::{http::HeaderMap, response::IntoResponse};
use opsml_storage::core::storage::base::UploadPartArgs;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResumableArgs {
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
        // get headers from the request
        let chunk_size = headers
            .get("Chunk-Size")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        let chunk_range = headers
            .get("Chunk-Range")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("0/0");

        let parts = chunk_range.split_once("/");

        // split the range to get the first and last chunk. Default to 0 if not found
        let (first_chunk, last_chunk) = match parts {
            Some(parts) => {
                let first = parts.0.parse::<u64>().unwrap_or(0);
                let last = parts.1.parse::<u64>().unwrap_or(0);
                (first, last)
            }
            None => (0, 0),
        };

        let part_number = headers
            .get("Part-Number")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let session_uri = headers
            .get("Session-Uri")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        let rpath = headers
            .get("Rpath")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        UploadPartArgs {
            first_chunk,
            last_chunk,
            chunk_size,
            part_number,
            session_uri,
            rpath,
        }
    }
}

use axum::Json;
use axum::{http::HeaderMap, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResumableArgs {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResumableId {
    pub upload_uri: String,
}

// Implement IntoResponse for Alive
impl IntoResponse for ResumableId {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct UploadPartArgs {
    pub first_chunk: u64,
    pub last_chunk: u64,
    pub chunk_size: u64,
    pub part_index: usize,
    pub upload_uri: String,
}

impl UploadPartArgs {
    pub fn new(headers: HeaderMap) -> Self {
        // get headers from the request
        let chunk_size = headers
            .get("Chunk-Size")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
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

        let part_index = headers
            .get("Part-Index")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(0);

        let upload_uri = headers
            .get("Upload-Uri")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        UploadPartArgs {
            first_chunk,
            last_chunk,
            chunk_size: chunk_size as u64,
            part_index,
            upload_uri: upload_uri.to_string(),
        }
    }
}

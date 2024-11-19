use crate::core::files::schema::{ResumableArgs, ResumableSession, UploadPartArgParser};
use crate::core::state::AppState;
/// Route for debugging information
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tracing::error;

use axum::{
    extract::{Multipart, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};

/// Route for uploading a part of a file
///
/// Used in conjunction with create_resumable_upload and multipart uploads on the client side
pub async fn upload_part(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let args = UploadPartArgParser::to_args(headers);

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        // pass entire stream to storage client

        while let Some(chunk) = field.chunk().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Multipart error: {}", e) })),
            )
        })? {
            // pass chunk to storage client
            println!("{:?}", chunk.len());
        }
    }

    Ok(())
}

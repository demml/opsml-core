use crate::core::files::schema::{ResumableArgs, ResumableId, UploadPartArgs};
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

pub async fn create_resumable_upload(
    State(data): State<Arc<AppState>>,
    params: Query<ResumableArgs>,
) -> Result<ResumableId, (StatusCode, Json<serde_json::Value>)> {
    let path = params.path.clone();
    let uri = data
        .storage_client
        .create_multipart_upload(Path::new(&path))
        .await;

    match uri {
        Ok(result) => {
            let resumable_id = ResumableId { upload_uri: result };
            Ok(resumable_id)
        }

        Err(e) => {
            error!("Failed to create resumable upload {:?}", e);
            let json_response = json!({
                "status": "resumable error",
                "message": format!("{:?}", e)
            });
            Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json_response)))
        }
    }
}

/// Route for uploading a part of a file
///
/// Used in conjunction with create_resumable_upload and multipart uploads on the client side
pub async fn upload_part(
    State(data): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let args = UploadPartArgs::new(headers);

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        println!("Length of `{}` is {} bytes", name, data.len());
    }

    Ok(())
}

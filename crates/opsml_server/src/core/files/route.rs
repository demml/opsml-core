use crate::core::error::ServerError;
use crate::core::files::schema::{
    MultiPartQuery, MultiPartSession, PresignedQuery, PresignedUrl, UploadPartArgParser,
};
use crate::core::state::AppState;
use axum::{
    extract::{Multipart, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use opsml_storage::core::storage::enums::MultiPartUploader;
/// Route for debugging information
use serde_json::json;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::error;

pub async fn create_multipart_upload(
    State(state): State<Arc<AppState>>,
    params: Query<MultiPartQuery>,
) -> Result<MultiPartSession, (StatusCode, Json<serde_json::Value>)> {
    let path = Path::new(&params.path);

    let session_url = state
        .storage_client
        .create_multipart_upload(path)
        .await
        .map_err(|e| ServerError::Error(format!("Failed to create multipart upload: {}", e)));

    let session_url = match session_url {
        Ok(session_url) => session_url,
        Err(e) => {
            error!("Failed to create multipart upload: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            ));
        }
    };

    Ok(MultiPartSession { session_url })
}

pub async fn generate_presigned_url(
    State(state): State<Arc<AppState>>,
    params: Query<PresignedQuery>,
) -> Result<PresignedUrl, (StatusCode, Json<serde_json::Value>)> {
    let path = Path::new(&params.path);
    let for_multi_part = params.for_multi_part.unwrap_or(false);

    // for multi part uploads, we need to get the session url and part number
    if for_multi_part {
        let session_url = params
            .session_url
            .as_ref()
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "Missing session_uri" })),
                )
            })?
            .to_string();

        let part_number = params.part_number.ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": "Missing part_number" })),
            )
        })?;

        let url = state
            .storage_client
            .generate_presigned_url_for_part(part_number as i32, &params.path, session_url)
            .await
            .map_err(|e| ServerError::Error(format!("Failed to generate presigned url: {}", e)));

        let url = match url {
            Ok(url) => url,
            Err(e) => {
                error!("Failed to generate presigned url: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e })),
                ));
            }
        };

        return Ok(PresignedUrl { url });
    }

    let url = state
        .storage_client
        .generate_presigned_url(path, 600)
        .await
        .map_err(|e| ServerError::Error(format!("Failed to generate presigned url: {}", e)));

    let url = match url {
        Ok(url) => url,
        Err(e) => {
            error!("Failed to generate presigned url: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            ));
        }
    };

    Ok(PresignedUrl { url })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::setup::setup_components;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        Router,
    };

    use std::sync::Arc;
    use tower::ServiceExt; // for `oneshot` method
}

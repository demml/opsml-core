use crate::core::error::ServerError;
use crate::core::files::schema::{
    ListFileInfoResponse, ListFileQuery, ListFileResponse, MultiPartQuery, MultiPartSession,
    PresignedQuery, PresignedUrl,
};
use crate::core::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use axum::{routing::get, Router};
/// Route for debugging information
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info};

pub async fn create_multipart_upload(
    State(state): State<Arc<AppState>>,
    params: Query<MultiPartQuery>,
) -> Result<MultiPartSession, (StatusCode, Json<serde_json::Value>)> {
    let path = Path::new(&params.path);

    info!("Creating multipart upload for path: {}", path.display());

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
            .generate_presigned_url_for_part(part_number, &params.path, session_url)
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

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    params: Query<ListFileQuery>,
) -> Result<ListFileResponse, (StatusCode, Json<serde_json::Value>)> {
    let path = Path::new(&params.path);

    info!("Listing files for: {}", path.display());

    let files = state
        .storage_client
        .find(path)
        .await
        .map_err(|e| ServerError::Error(format!("Failed to list files: {}", e)));

    let files = match files {
        Ok(files) => files,
        Err(e) => {
            error!("Failed to list files: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            ));
        }
    };

    Ok(ListFileResponse { files })
}

pub async fn list_file_info(
    State(state): State<Arc<AppState>>,
    params: Query<ListFileQuery>,
) -> Result<ListFileInfoResponse, (StatusCode, Json<serde_json::Value>)> {
    let path = Path::new(&params.path);

    info!("Getting file info for: {}", path.display());

    let files = state
        .storage_client
        .find_info(path)
        .await
        .map_err(|e| ServerError::Error(format!("Failed to list files: {}", e)));

    let files = match files {
        Ok(files) => files,
        Err(e) => {
            error!("Failed to list files: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            ));
        }
    };

    Ok(ListFileInfoResponse { files })
}

pub async fn get_file_router(prefix: &str) -> Router<Arc<AppState>> {
    Router::new()
        .route(
            &format!("{}/files/multipart", prefix),
            get(create_multipart_upload),
        )
        .route(
            &format!("{}/files/presigned", prefix),
            get(generate_presigned_url),
        )
        .route(&format!("{}/files/list", prefix), get(list_files))
        .route(&format!("{}/files/list/info", prefix), get(list_file_info))
}

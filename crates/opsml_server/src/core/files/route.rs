use crate::core::error::internal_server_error;
use crate::core::files::schema::{DeleteFileQuery, ListFileQuery, MultiPartQuery, PresignedQuery};
use crate::core::state::AppState;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use axum::{
    routing::{delete, get},
    Router,
};
use opsml_contracts::{
    DeleteFileResponse, ListFileInfoResponse, ListFileResponse, MultiPartSession, PresignedUrl,
};

use opsml_error::error::ServerError;
/// Route for debugging information
use serde_json::json;
use std::path::Path;
use std::sync::Arc;
use tracing::{error, info};

pub async fn create_multipart_upload(
    State(state): State<Arc<AppState>>,
    params: Query<MultiPartQuery>,
) -> Result<Json<MultiPartSession>, (StatusCode, Json<serde_json::Value>)> {
    let path = Path::new(&params.path);

    info!("Creating multipart upload for path: {}", path.display());

    let session_url = state
        .storage_client
        .create_multipart_upload(path)
        .await
        .map_err(|e| ServerError::MultipartError(e.to_string()));

    let session_url = match session_url {
        Ok(session_url) => session_url,
        Err(e) => {
            error!("Failed to create multipart upload: {}", e);
            return Err(internal_server_error(e));
        }
    };

    Ok(Json(MultiPartSession { session_url }))
}

pub async fn generate_presigned_url(
    State(state): State<Arc<AppState>>,
    params: Query<PresignedQuery>,
) -> Result<Json<PresignedUrl>, (StatusCode, Json<serde_json::Value>)> {
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
            .map_err(|e| ServerError::PresignedError(e.to_string()));

        let url = match url {
            Ok(url) => url,
            Err(e) => {
                error!("Failed to generate presigned url: {}", e);
                return Err(internal_server_error(e));
            }
        };

        return Ok(Json(PresignedUrl { url }));
    }

    let url = state
        .storage_client
        .generate_presigned_url(path, 600)
        .await
        .map_err(|e| ServerError::PresignedError(e.to_string()));

    let url = match url {
        Ok(url) => url,
        Err(e) => {
            error!("Failed to generate presigned url: {}", e);
            return Err(internal_server_error(e));
        }
    };

    Ok(Json(PresignedUrl { url }))
}

pub async fn list_files(
    State(state): State<Arc<AppState>>,
    params: Query<ListFileQuery>,
) -> Result<Json<ListFileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let path = Path::new(&params.path);

    info!("Listing files for: {}", path.display());

    let files = state
        .storage_client
        .find(path)
        .await
        .map_err(|e| ServerError::ListFileError(e.to_string()));

    let files = match files {
        Ok(files) => files,
        Err(e) => {
            error!("Failed to list files: {}", e);
            return Err(internal_server_error(e));
        }
    };

    Ok(Json(ListFileResponse { files }))
}

pub async fn list_file_info(
    State(state): State<Arc<AppState>>,
    params: Query<ListFileQuery>,
) -> Result<Json<ListFileInfoResponse>, (StatusCode, Json<serde_json::Value>)> {
    let path = Path::new(&params.path);

    info!("Getting file info for: {}", path.display());

    let files = state
        .storage_client
        .find_info(path)
        .await
        .map_err(|e| ServerError::ListFileError(e.to_string()));

    let files = match files {
        Ok(files) => files,
        Err(e) => {
            error!("Failed to list files: {}", e);
            return Err(internal_server_error(e));
        }
    };

    Ok(Json(ListFileInfoResponse { files }))
}

pub async fn delete_file(
    State(state): State<Arc<AppState>>,
    params: Query<DeleteFileQuery>,
) -> Result<Json<DeleteFileResponse>, (StatusCode, Json<serde_json::Value>)> {
    let path = Path::new(&params.path);
    let recursive = params.recursive;

    info!("Deleting path: {}", path.display());

    let files = state.storage_client.rm(path, recursive).await.map_err(|e| {
        error!("Failed to delete files: {}", e);
        ServerError::DeleteError(e.to_string())
    });

    //
    if let Err(e) = files {
        return Err(internal_server_error(e));
    }

    // check if file exists
    let exists = state.storage_client.exists(path).await;

    match exists {
        Ok(exists) => {
            if exists {
                return Err(internal_server_error("Failed to delete file"));
            } else {
                return Ok(Json(DeleteFileResponse { deleted: true }));
            }
        }
        Err(e) => {
            error!("Failed to check if file exists: {}", e);
            return Err(internal_server_error(e));
        }
    }
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
        .route(&format!("{}/files/delete", prefix), delete(delete_file))
}

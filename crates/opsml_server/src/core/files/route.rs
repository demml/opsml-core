use crate::core::error::internal_server_error;
use crate::core::files::schema::{
    DeleteFileQuery, DownloadFileQuery, ListFileQuery, MultiPartQuery, PresignedQuery,
};
use crate::core::state::AppState;
use axum::extract::{Path as AxumPath, Request};
use axum::response::IntoResponse;
use axum::response::Response;
use axum::BoxError;
use axum::{
    body::Body,
    routing::{delete, get, post},
    Router,
};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use bytes::Bytes;
use futures::{Stream, TryStreamExt};
use opsml_contracts::{
    DeleteFileResponse, ListFileInfoResponse, ListFileResponse, MultiPartSession, PresignedUrl,
    UploadResponse,
};
use opsml_settings::config::StorageType;
use std::io;
use std::path::{Path, PathBuf};
use tokio_util::io::StreamReader;

use tokio::fs::File;
use tokio::io::BufWriter;
use tokio_util::io::ReaderStream;

use anyhow::{Context, Result};
use opsml_error::error::ServerError;

/// Route for debugging information
use serde_json::json;
use std::panic::{catch_unwind, AssertUnwindSafe};
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

        let path = Path::new(&params.path);
        let url = state
            .storage_client
            .generate_presigned_url_for_part(part_number, path, session_url)
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

// this is for local storage only
async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    async {
        let path = Path::new(path);

        // create the file parents if they don't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
                .unwrap();
        }

        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

// Handler that streams the request body to a file.
//
// POST'ing to `/file/foo.txt` will create a file called `foo.txt`.
async fn save_request_body(request: Request) -> Result<(), (StatusCode, String)> {
    // get filename header
    let headers = request.headers().clone();
    let file_name = headers
        .get("File-Name")
        .ok_or((StatusCode::BAD_REQUEST, "Missing filename header"));

    let filename = match file_name {
        Ok(file_name) => file_name.to_str().unwrap(),
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                "Missing filename header".to_string(),
            ))
        }
    };

    stream_to_file(filename, request.into_body().into_data_stream()).await
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
                Err(internal_server_error("Failed to delete file"))
            } else {
                Ok(Json(DeleteFileResponse { deleted: true }))
            }
        }
        Err(e) => {
            error!("Failed to check if file exists: {}", e);
            Err(internal_server_error(e))
        }
    }
}

pub async fn download_file(
    State(state): State<Arc<AppState>>,
    params: Query<DownloadFileQuery>,
) -> Response<Body> {
    // check if storage client is local (fails if not)
    if state.storage_client.storage_type() != StorageType::Local {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Download is only supported for local storage" })),
        )
            .into_response();
    }

    let file = match File::open(&params.path).await {
        Ok(file) => file,
        Err(e) => {
            error!("Failed to open file: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "failed": "Failed to open file" })),
            )
                .into_response();
        }
    };

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    (StatusCode::OK, body).into_response()
}

pub async fn get_file_router(prefix: &str) -> Result<Router<Arc<AppState>>> {
    let result = catch_unwind(AssertUnwindSafe(|| {
        Router::new()
            .route(
                &format!("{}/files/multipart", prefix),
                get(create_multipart_upload),
            )
            .route(&format!("{}/files", prefix), post(save_request_body))
            .route(
                &format!("{}/files/presigned", prefix),
                get(generate_presigned_url),
            )
            .route(&format!("{}/files", prefix), get(download_file))
            .route(&format!("{}/files/list", prefix), get(list_files))
            .route(&format!("{}/files/list/info", prefix), get(list_file_info))
            .route(&format!("{}/files/delete", prefix), delete(delete_file))
    }));

    match result {
        Ok(router) => Ok(router),
        Err(_) => {
            error!("Failed to create file router");
            // panic
            Err(anyhow::anyhow!("Failed to create file router"))
                .context("Panic occurred while creating the router")
        }
    }
}

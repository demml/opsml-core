use crate::core::files::schema::UploadPartArgParser;
use crate::core::state::AppState;
/// Route for debugging information
use serde_json::json;
use std::sync::Arc;
use tracing::error;

use crate::core::error::ServerError;
use axum::{
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    Json,
};

/// Route for uploading a part of a file
///
/// Used in conjunction with create_resumable_upload and multipart uploads on the client side
pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let args = UploadPartArgParser::to_args(headers);

    while let Some(mut field) = multipart.next_field().await.unwrap() {
        // get multipart uploader
        let path = &args.path;

        let uploader = state
            .storage_client
            .create_multipart_upload(path)
            .await
            .map_err(|e| ServerError::Error(format!("Failed to create multipart upload: {}", e)));

        let mut uploader = match uploader {
            Ok(uploader) => uploader,
            Err(e) => {
                error!("Failed to create multipart upload: {}", e);
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": e })),
                ));
            }
        };

        let mut bytes_stream = bytes::BytesMut::new();
        let mut first_byte = 0;
        let mut part_number = 1;

        while let Some(chunk) = field.chunk().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Multipart error: {}", e) })),
            )
        })? {
            // Process the chunk in the specified size
            bytes_stream.extend_from_slice(&chunk);

            // If the stream is greater than the chunk size, upload it
            if bytes_stream.len() >= args.chunk_size as usize {
                // calculate the first and last byte

                let last_byte = first_byte + (bytes_stream.len() - 1) as u64;

                uploader
                    .upload_part(
                        &first_byte,
                        &last_byte,
                        &part_number,
                        &args.file_size,
                        bytes_stream.split().freeze(),
                    )
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "error": format!("Failed to upload part: {}", e) })),
                        )
                    })?;

                // Reinitialize bytes_stream after each upload
                bytes_stream = bytes::BytesMut::new();

                // Increment the part number and first byte
                part_number += 1;
                first_byte = last_byte + 1;
            }
        } //

        // Upload the remaining bytes
        if !bytes_stream.is_empty() {
            let last_byte = first_byte + (bytes_stream.len() - 1) as u64;

            uploader
                .upload_part(
                    &first_byte,
                    &last_byte,
                    &part_number,
                    &args.file_size,
                    bytes_stream.split().freeze(),
                )
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": format!("Failed to upload part: {}", e) })),
                    )
                })?;
        }

        // Complete the upload
        uploader.complete_upload().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to complete upload: {}", e) })),
            )
        })?;
    }

    Ok(())
}

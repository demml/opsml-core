use crate::core::state::AppState;
use anyhow::{Context, Result};
/// Route for debugging information
use axum::extract::State;
use axum::{http::header, http::header::HeaderMap, http::StatusCode, routing::get, Json, Router};
use opsml_sql::base::SqlClient;
use opsml_types::CardSQLTableNames;
use opsml_types::{UidRequest, UidResponse};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use tracing::error;

/// Route for checking if a card UID exists
pub async fn check_card_uid(
    State(state): State<Arc<AppState>>,
    uid_request: UidRequest,
) -> Result<Json<UidResponse>, (StatusCode, Json<serde_json::Value>)> {
    let table = CardSQLTableNames::from_registry_type(&uid_request.registry_type);
    let exists = state
        .sql_client
        .check_uid_exists(&uid_request.uid, &table)
        .await
        .map_err(|e| {
            error!("Failed to check if UID exists: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({})),
            )
        })?;

    Ok(Json(UidResponse { exists: exists }))
}

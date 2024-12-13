use crate::core::state::AppState;
use anyhow::{Context, Result};
/// Route for debugging information
use axum::extract::State;
use axum::{http::header, http::header::HeaderMap, http::StatusCode, routing::get, Json, Router};
use opsml_sql::base::SqlClient;
use opsml_types::types::JwtToken;
use opsml_types::{UidRequest, UidResponse};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use tracing::error;

pub async fn check_card_uid(
    State(state): State<Arc<AppState>>,
    uid_request: UidRequest,
) -> Result<Json<UidResponse>, (StatusCode, Json<serde_json::Value>)> {
    state.Ok(Json(UidResponse { exists: true }))
}

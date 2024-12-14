/// Route for checking if a card UID exists
use crate::core::cards::schema::{QueryPageResponse, RegistryStatsResponse};
use crate::core::state::AppState;
use anyhow::{Context, Result};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use opsml_sql::base::SqlClient;
use opsml_sql::schemas::schema::{
    AuditCardRecord, Card, CardResults, DataCardRecord, MetricRecord, ModelCardRecord,
    PipelineCardRecord, ProjectCardRecord, RunCardRecord,
};
use opsml_types::{CardSQLTableNames, MetricRequest, UidResponse};
use opsml_utils::semver::{VersionArgs, VersionValidator};
use semver::Version;
use sqlx::types::Json as SqlxJson;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use tracing::error;

pub async fn insert_metrics(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MetricRequest>,
) -> Result<Json<UidResponse>, (StatusCode, Json<serde_json::Value>)> {
    let records = req
        .metrics
        .iter()
        .map(|m| {
            MetricRecord::new(
                req.run_uid.clone(),
                m.name.clone(),
                m.value,
                m.step,
                m.timestamp,
            )
        })
        .collect::<Vec<_>>();

    let exists = state
        .sql_client
        .insert_run_metrics(&records)
        .await
        .map_err(|e| {
            error!("Failed to insert metric: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({})),
            )
        })?;

    Ok(Json(UidResponse { exists: true }))
}

use crate::core::state::AppState;
use anyhow::{Context, Result};
use axum::extract::{Query, State};
use axum::{http::StatusCode, routing::get, Json, Router};
use opsml_sql::base::SqlClient;
use opsml_types::CardSQLTableNames;
use opsml_types::{RepositoryRequest, RepositoryResponse, UidRequest, UidResponse};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use tracing::error;

/// Route for checking if a card UID exists
pub async fn check_card_uid(
    State(state): State<Arc<AppState>>,
    params: Query<UidRequest>,
) -> Result<Json<UidResponse>, (StatusCode, Json<serde_json::Value>)> {
    let table = CardSQLTableNames::from_registry_type(&params.registry_type);
    let exists = state
        .sql_client
        .check_uid_exists(&params.uid, &table)
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

/// Get card respositories
pub async fn get_card_repositories(
    State(state): State<Arc<AppState>>,
    params: Query<RepositoryRequest>,
) -> Result<Json<RepositoryResponse>, (StatusCode, Json<serde_json::Value>)> {
    let table = CardSQLTableNames::from_registry_type(&params.registry_type);
    let repos = state
        .sql_client
        .get_unique_repository_names(&table)
        .await
        .map_err(|e| {
            error!("Failed to get unique repository names: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({})),
            )
        })?;

    Ok(Json(RepositoryResponse {
        repositories: repos,
    }))
}

pub async fn get_card_router(prefix: &str) -> Result<Router<Arc<AppState>>> {
    let result = catch_unwind(AssertUnwindSafe(|| {
        Router::new()
            .route(&format!("{}/card", prefix), get(check_card_uid))
            .route(
                &format!("{}/card/repositories", prefix),
                get(get_card_repositories),
            )
    }));

    match result {
        Ok(router) => Ok(router),
        Err(_) => {
            error!("Failed to create card router");
            // panic
            Err(anyhow::anyhow!("Failed to create card router"))
                .context("Panic occurred while creating the router")
        }
    }
}

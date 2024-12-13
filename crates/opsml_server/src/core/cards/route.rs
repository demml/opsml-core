use crate::core::cards::schema::{QueryPageResponse, RegistryStatsResponse};
use crate::core::state::AppState;
use anyhow::{Context, Result};
use axum::extract::{Query, State};
use axum::{http::StatusCode, routing::get, Json, Router};
use opsml_sql::base::SqlClient;
use opsml_types::{
    CardSQLTableNames, CardVersionRequest, CardVersionResponse, QueryPageRequest,
    RegistryStatsRequest, RepositoryRequest, RepositoryResponse, UidRequest, UidResponse,
};
use opsml_utils::semver::{VersionArgs, VersionValidator};
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

    Ok(Json(UidResponse { exists }))
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

/// query stats page
pub async fn get_registry_stats(
    State(state): State<Arc<AppState>>,
    params: Query<RegistryStatsRequest>,
) -> Result<Json<RegistryStatsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let table = CardSQLTableNames::from_registry_type(&params.registry_type);
    let stats = state
        .sql_client
        .query_stats(&table, params.search_term.as_deref())
        .await
        .map_err(|e| {
            error!("Failed to get unique repository names: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({})),
            )
        })?;

    Ok(Json(RegistryStatsResponse { stats }))
}

// query page
pub async fn get_page(
    State(state): State<Arc<AppState>>,
    params: Query<QueryPageRequest>,
) -> Result<Json<QueryPageResponse>, (StatusCode, Json<serde_json::Value>)> {
    let table = CardSQLTableNames::from_registry_type(&params.registry_type);
    let sort_by = &params.sort_by.clone().unwrap_or("updated_at".to_string());
    let page = params.page.unwrap_or(0);
    let summaries = state
        .sql_client
        .query_page(
            sort_by,
            page,
            params.search_term.as_deref(),
            params.repository.as_deref(),
            &table,
        )
        .await
        .map_err(|e| {
            error!("Failed to get unique repository names: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({})),
            )
        })?;

    Ok(Json(QueryPageResponse { summaries }))
}

pub async fn get_next_version(
    State(state): State<Arc<AppState>>,
    params: Query<CardVersionRequest>,
) -> Result<Json<CardVersionResponse>, (StatusCode, Json<serde_json::Value>)> {
    let table = CardSQLTableNames::from_registry_type(&params.registry_type);
    let version_type = params.version_type.clone();
    let pre_tag = params.pre_tag.clone();
    let build_tag = params.build_tag.clone();

    let versions = state
        .sql_client
        .get_versions(
            &table,
            &params.name,
            &params.repository,
            params.version.as_deref(),
        )
        .await
        .map_err(|e| {
            error!("Failed to get unique repository names: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({})),
            )
        })?;
    let version = versions.first().ok_or_else(|| {
        error!("Failed to get first version");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({})),
        )
    })?;

    let args = VersionArgs {
        version: version.to_string(),
        version_type,
        pre: pre_tag,
        build: build_tag,
    };

    let bumped_version = VersionValidator::bump_version(&args).map_err(|e| {
        error!("Failed to bump version: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({})),
        )
    })?;

    Ok(Json(CardVersionResponse {
        version: bumped_version,
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
            .route(
                &format!("{}/card/registry/stats", prefix),
                get(get_registry_stats),
            )
            .route(&format!("{}/card/registry/page", prefix), get(get_page))
            .route(&format!("{}/card/version", prefix), get(get_next_version))
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

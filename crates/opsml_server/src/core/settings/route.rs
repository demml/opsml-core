use crate::core::state::AppState;
/// Route for debugging information
use axum::extract::State;
use axum::Json;
use axum::{routing::get, Router};
use opsml_contracts::StorageSettings;
use std::sync::Arc;

pub async fn storage_settings(State(data): State<Arc<AppState>>) -> Json<StorageSettings> {
    Json(StorageSettings {
        storage_type: data.storage_client.storage_type(),
    })
}

pub async fn get_settings_router(prefix: &str) -> Router<Arc<AppState>> {
    Router::new().route(
        &format!("{}/storage/settings", prefix),
        get(storage_settings),
    )
}

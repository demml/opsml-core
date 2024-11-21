use crate::core::settings::schema::StorageSettings;
use crate::core::state::AppState;
/// Route for debugging information
use axum::extract::State;
use axum::{routing::get, Router};
use std::sync::Arc;

pub async fn storage_settings(State(data): State<Arc<AppState>>) -> StorageSettings {
    StorageSettings {
        storage_type: data.storage_client.storage_type(),
    }
}

pub async fn get_settings_router(prefix: &str) -> Router<Arc<AppState>> {
    Router::new().route(
        &format!("{}/storage/settings", prefix),
        get(storage_settings),
    )
}

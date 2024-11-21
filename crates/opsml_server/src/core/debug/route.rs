use crate::core::debug::schema::DebugInfo;
use crate::core::state::AppState;
/// Route for debugging information
use axum::extract::State;
use axum::{routing::get, Router};
use std::sync::Arc;

pub async fn debug_info(State(data): State<Arc<AppState>>) -> DebugInfo {
    DebugInfo::new(
        data.storage_client.name().to_string(),
        data.config.opsml_storage_uri.clone(),
        data.config.opsml_tracking_uri.clone(),
    )
}

pub async fn get_debug_router(prefix: &str) -> Router<Arc<AppState>> {
    Router::new().route(&format!("{}/debug", prefix), get(debug_info))
}

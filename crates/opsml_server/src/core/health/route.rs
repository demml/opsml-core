use crate::core::health::schema::Alive;
use crate::core::state::AppState;
use axum::{routing::get, Router};
use std::sync::Arc;

pub async fn health_check() -> Alive {
    Alive::default()
}

pub async fn get_health_router(prefix: &str) -> Router<Arc<AppState>> {
    Router::new().route(&format!("{}/healthcheck", prefix), get(health_check))
}

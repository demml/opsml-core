use crate::core::auth::route::get_auth_router;
use crate::core::debug::route::get_debug_router;
use crate::core::files::route::get_file_router;
use crate::core::health::route::get_health_router;
use crate::core::settings::route::get_settings_router;
use crate::core::state::AppState;
use anyhow::Result;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Method,
};
use axum::Router;
use opsml_settings::config::OpsmlAuthSettings;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

const ROUTE_PREFIX: &str = "/opsml";

pub async fn create_router(auth: &OpsmlAuthSettings, app_state: Arc<AppState>) -> Result<Router> {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::PUT,
            Method::DELETE,
            Method::POST,
            Method::PATCH,
        ])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let debug_routes = get_debug_router(ROUTE_PREFIX).await?;
    let health_routes = get_health_router(auth, &app_state, ROUTE_PREFIX).await?;
    let file_routes = get_file_router(auth, &app_state, ROUTE_PREFIX).await?;
    let settings_routes = get_settings_router(ROUTE_PREFIX).await?;
    let auth_routes = get_auth_router(ROUTE_PREFIX).await?;

    Ok(Router::new()
        .merge(debug_routes)
        .merge(health_routes)
        .merge(settings_routes)
        .merge(file_routes)
        .merge(auth_routes)
        .layer(cors)
        .with_state(app_state))
}

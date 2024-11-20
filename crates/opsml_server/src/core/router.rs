use crate::core::debug::route::debug_info;
use crate::core::files::route::{create_multipart_upload, generate_presigned_url};
use crate::core::health::route::health_check;
use crate::core::state::AppState;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    Method,
};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

const ROUTE_PREFIX: &str = "/opsml";

pub async fn create_router(app_state: Arc<AppState>) -> Router {
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

    Router::new()
        .route(&format!("{}/healthcheck", ROUTE_PREFIX), get(health_check))
        .route(&format!("{}/debug", ROUTE_PREFIX), get(debug_info))
        .route(
            &format!("{}/files/multipart", ROUTE_PREFIX),
            get(create_multipart_upload),
        )
        .route(
            &format!("{}/files/presigned", ROUTE_PREFIX),
            get(generate_presigned_url),
        )
        .with_state(app_state)
        .layer(cors)
}

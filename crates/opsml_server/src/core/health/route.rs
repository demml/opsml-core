use crate::core::auth::middleware::auth_api_middleware;
use crate::core::auth::middleware::JWTAuthMiddleware;
use crate::core::health::schema::Alive;
use crate::core::state::AppState;
use anyhow::{Context, Result};
use axum::middleware;
use axum::{routing::get, Extension, Router};
use opsml_settings::config::OpsmlAuthSettings;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use tracing::info;

pub async fn health_check(Extension(auth): Extension<Option<JWTAuthMiddleware>>) -> Alive {
    if let Some(_auth) = auth {
        info!("✅ Auth enabled for health check");
    }
    Alive::default()
}

pub async fn get_health_router(
    opsml_auth: &OpsmlAuthSettings,
    app_state: &Arc<AppState>,
    prefix: &str,
) -> Result<Router<Arc<AppState>>> {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let mut router = Router::new().route(&format!("{}/healthcheck", prefix), get(health_check));

        if opsml_auth.enabled {
            info!("✅ Auth enabled for health routes");
            router = router.route_layer(middleware::from_fn_with_state(
                app_state.clone(),
                auth_api_middleware,
            ));
        }
        router
    }));

    match result {
        Ok(router) => Ok(router),
        Err(_) => {
            // panic
            Err(anyhow::anyhow!("Failed to create health router"))
                .context("Panic occurred while creating the router")
        }
    }
}

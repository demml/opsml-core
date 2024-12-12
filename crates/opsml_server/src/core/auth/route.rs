use crate::core::auth::schema::JwtToken;
use crate::core::state::AppState;
use anyhow::{Context, Result};
/// Route for debugging information
use axum::extract::State;
use axum::{http::header::HeaderMap, http::StatusCode, routing::get, Json, Router};
use opsml_sql::base::SqlClient;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;
use tracing::error;

pub async fn api_login_handler(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<JwtToken>, (StatusCode, Json<serde_json::Value>)> {
    // get Username and Password from headers
    let username = headers
        .get("Username")
        .expect("Username not found in headers")
        .to_str()
        .expect("Failed to convert Username to string")
        .to_string();

    let password = headers
        .get("Password")
        .expect("Password not found in headers")
        .to_str()
        .expect("Failed to convert Password to string")
        .to_string();

    // get user from database
    let mut user = state.sql_client.get_user(&username).await.map_err(|e| {
        error!("Failed to get user from database: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({})),
        )
    })?;

    // check if password is correct
    state
        .auth_manager
        .validate_user(&user, &password)
        .map_err(|e| {
            error!("Failed to validate user: {}", e);
            (StatusCode::UNAUTHORIZED, Json(serde_json::json!({})))
        })?;

    // generate JWT token
    let jwt_token = state.auth_manager.generate_jwt(&user);
    let refresh_token = state.auth_manager.generate_refresh_token(&user);

    user.refresh_token = Some(refresh_token);

    // set refresh token in db
    state.sql_client.update_user(&user).await.map_err(|e| {
        error!("Failed to set refresh token in database: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({})),
        )
    })?;

    Ok(Json(JwtToken { token: jwt_token }))
}

pub async fn get_auth_router(prefix: &str) -> Result<Router<Arc<AppState>>> {
    let result = catch_unwind(AssertUnwindSafe(|| {
        Router::new().route(&format!("{}/auth/login", prefix), get(api_login_handler))
    }));

    match result {
        Ok(router) => Ok(router),
        Err(_) => {
            error!("Failed to create sauth router");
            // panic
            Err(anyhow::anyhow!("Failed to create auth router"))
                .context("Panic occurred while creating the router")
        }
    }
}

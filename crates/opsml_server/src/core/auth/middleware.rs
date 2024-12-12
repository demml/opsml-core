use crate::core::auth::schema::AuthError;
use crate::core::state::AppState;
use axum::extract::FromRequestParts;
use axum::http::header;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{
    body::Body,
    extract::{Request, State},
    middleware::Next,
    response::{Json, Response},
};
use axum_extra::extract::cookie::CookieJar;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

pub async fn auth_api_middleware(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<AuthError>)> {
    let access_token = cookie_jar
        .get("access_token")
        .map(|cookie| cookie.value().to_string())
        .or_else(|| {
            req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|auth_header| auth_header.to_str().ok())
                .and_then(|auth_value| {
                    if auth_value.starts_with("Bearer ") {
                        Some(auth_value[7..].to_owned())
                    } else {
                        None
                    }
                })
        });

    let access_token = access_token.ok_or_else(|| {
        (
            StatusCode::UNAUTHORIZED,
            Json(AuthError {
                error: "Unauthorized".to_string(),
                message: "No access token provided".to_string(),
            }),
        )
    })?;

    // validate the access token (this will also check if the token is expired)
    match state.auth_manager.validate_jwt(&access_token) {
        Ok(claims) => claims,
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(AuthError {
                    error: "Unauthorized".to_string(),
                    message: "No refresh token found".to_string(),
                }),
            ));
        }
    };

    Ok(next.run(req).await)
}

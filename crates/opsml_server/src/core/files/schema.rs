use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ResumableArgs {
    pub path: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResumableId {
    pub upload_uri: String,
}

// Implement IntoResponse for Alive
impl IntoResponse for ResumableId {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

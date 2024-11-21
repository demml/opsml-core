use axum::response::IntoResponse;
use axum::Json;
/// file containing schema for health module
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageSettings {
    pub storage_type: String,
}

// Implement IntoResponse for Alive
impl IntoResponse for StorageSettings {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

use axum::response::IntoResponse;
use axum::Json;
use opsml_settings::config::StorageType;
/// file containing schema for health module
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct StorageSettings {
    pub storage_type: StorageType,
}

// Implement IntoResponse for Alive
impl IntoResponse for StorageSettings {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}

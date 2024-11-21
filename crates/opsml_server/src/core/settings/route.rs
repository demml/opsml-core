use crate::core::settings::schema::StorageSettings;
use crate::core::state::AppState;
/// Route for debugging information
use axum::extract::State;
use std::sync::Arc;

pub async fn storage_settings(State(data): State<Arc<AppState>>) -> StorageSettings {
    StorageSettings {
        storage_type: data.storage_client.storage_type(),
    }
}

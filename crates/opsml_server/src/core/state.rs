use crate::core::storage::client::StorageClientEnum;
use opsml_settings::config::OpsmlConfig;
use std::sync::Arc;

pub struct AppState {
    pub storage_client: Arc<StorageClientEnum>,
    pub config: Arc<OpsmlConfig>,
}

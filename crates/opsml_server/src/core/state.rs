use opsml_settings::config::OpsmlConfig;
use opsml_storage::storage::enums::client::StorageClientEnum;
use std::sync::Arc;

pub struct AppState {
    pub storage_client: Arc<StorageClientEnum>,
    pub config: Arc<OpsmlConfig>,
}

impl AppState {
    pub fn new(storage_client: StorageClientEnum, config: OpsmlConfig) -> Self {
        Self {
            storage_client: Arc::new(storage_client),
            config: Arc::new(config),
        }
    }
}

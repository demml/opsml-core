use opsml_settings::config::OpsmlConfig;
use opsml_storage::storage::enums::client::StorageClientEnum;
use std::sync::Arc;

pub struct AppState {
    pub storage_client: Arc<StorageClientEnum>,
    pub config: Arc<OpsmlConfig>,
}

use crate::core::storage::client::StorageClientEnum;
use std::sync::Arc;

use super::setup::OpsmlConfig;

pub struct AppState {
    pub storage_client: Arc<StorageClientEnum>,
    pub config: Arc<OpsmlConfig>,
}

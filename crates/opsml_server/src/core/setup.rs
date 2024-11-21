use crate::core::logging::route::setup_logging;

use opsml_settings::config::OpsmlConfig;
use opsml_storage::core::storage::base::get_storage_system;
use opsml_storage::core::storage::enums::StorageClientEnum;
use tracing::info;

pub async fn setup_components() -> Result<(OpsmlConfig, StorageClientEnum), anyhow::Error> {
    // setup config
    let config = OpsmlConfig::default();

    // start logging
    setup_logging().await?;

    info!("Starting OpsML Server ....");

    // setup storage client
    let storage = get_storage_system(&config).await?;

    info!("Storage client: {}", storage.name());

    Ok((config, storage))
}

use anyhow::{Context, Result as AnyhowResult};
use opsml_logging::logging::setup_logging;
use opsml_settings::config::OpsmlConfig;
use opsml_storage::storage::enums::client::{get_storage_system, StorageClientEnum};
use opsml_utils::color::LogColors;
use tracing::info;

pub async fn setup_components() -> AnyhowResult<(OpsmlConfig, StorageClientEnum)> {
    // setup config
    let config = OpsmlConfig::default();

    // start logging
    setup_logging()
        .await
        .context(LogColors::purple("Failed to setup logging"))?;

    info!("Starting OpsML Server ....");

    // setup storage client
    let storage = get_storage_system(&config)
        .await
        .context(LogColors::purple("Failed to setup storage client"))?;

    info!("Storage client: {}", storage.name());

    Ok((config, storage))
}

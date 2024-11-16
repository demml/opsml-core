use crate::core::logging::route::setup_logging;
use anyhow::Context;
use opsml_settings::config::OpsmlConfig;
use opsml_storage::core::storage::enums::StorageClientEnum;

pub async fn get_storage_system(storage_uri: &str) -> Result<StorageClientEnum, anyhow::Error> {
    StorageClientEnum::new(storage_uri.to_string())
        .await
        .with_context(|| format!("Failed to create storage client for uri: {}", storage_uri))
}

pub async fn setup_components() -> Result<(OpsmlConfig, StorageClientEnum), anyhow::Error> {
    // setup config
    let config = OpsmlConfig::default();

    // start logging
    setup_logging().await?;

    // setup storage client
    let storage = get_storage_system(&config.opsml_storage_uri).await?;

    Ok((config, storage))
}

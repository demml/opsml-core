use crate::core::logging::route::setup_logging;
use anyhow::Context;
use opsml_settings::config::OpsmlConfig;
use opsml_storage::core::storage::base::{StorageSettings, StorageType};
use opsml_storage::core::storage::enums::StorageClientEnum;

pub async fn get_storage_system(config: &OpsmlConfig) -> Result<StorageClientEnum, anyhow::Error> {
    // check storage_uri for prefix
    let storage_type = if config.opsml_storage_uri.starts_with("gs://") {
        StorageType::Google
    } else if config.opsml_storage_uri.starts_with("s3://") {
        StorageType::AWS
    } else {
        StorageType::Local
    };

    // we don't use http
    let settings = StorageSettings::new(
        config.opsml_storage_uri.clone(),
        config.is_using_client(),
        storage_type,
        Default::default(),
    );

    StorageClientEnum::new(settings).await.with_context(|| {
        format!(
            "Failed to create storage client for uri: {}",
            config.opsml_storage_uri
        )
    })
}

pub async fn setup_components() -> Result<(OpsmlConfig, StorageClientEnum), anyhow::Error> {
    // setup config
    let config = OpsmlConfig::default();

    // start logging
    setup_logging().await?;

    // setup storage client
    let storage = get_storage_system(&config).await?;

    println!("name: {}", storage.name());

    Ok((config, storage))
}

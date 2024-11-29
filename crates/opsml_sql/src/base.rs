use async_trait::async_trait;
use opsml_settings::config::OpsmlStorageSettings;

#[async_trait]
pub trait FileSystem {
    async fn new(settings: &OpsmlStorageSettings) -> Self;
}

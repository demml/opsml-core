use async_trait::async_trait;
use opsml_settings::config::OpsmlDatabaseSettings;

#[async_trait]
pub trait SqlClient {
    async fn new(settings: &OpsmlDatabaseSettings) -> Self;
}

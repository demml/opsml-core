use opsml_error::error::RegistryError;
use opsml_settings::config::OpsmlStorageSettings;
use opsml_storage::HttpFSStorageClient;
use opsml_types::*;

// TODO: Add trait for client and server registry
pub struct ClientRegistry {
    registry_type: RegistryType,
    api_client: HttpFSStorageClient,
}

impl ClientRegistry {
    pub async fn new(
        storage_settings: &mut OpsmlStorageSettings,
        registry_type: RegistryType,
    ) -> Result<Self, RegistryError> {
        let api_client = HttpFSStorageClient::new(storage_settings)
            .await
            .map_err(|e| RegistryError::NewError(format!("Failed to create http client {}", e)))?;
        Ok(Self {
            registry_type,
            api_client,
        })
    }

    pub fn table_name(&self) -> String {
        CardSQLTableNames::from_registry_type(&self.registry_type).to_string()
    }
}

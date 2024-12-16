use opsml_error::error::RegistryError;
use opsml_settings::config::OpsmlStorageSettings;
use opsml_storage::*;
use opsml_types::*;

// TODO: Add trait for client and server registry
pub struct ClientRegistry {
    registry_type: RegistryType,
    api_client: OpsmlApiClient,
}

impl ClientRegistry {
    pub async fn new(
        storage_settings: &OpsmlStorageSettings,
        registry_type: RegistryType,
    ) -> Result<Self, RegistryError> {
        let client = build_http_client(&storage_settings.api_settings)
            .map_err(|e| RegistryError::NewError(format!("Failed to create http client {}", e)))?;

        let api_client = OpsmlApiClient::new(&storage_settings, &client)
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

    pub async fn list_cards(&self, args: CardQueryArgs) -> Result<Vec<Card>, RegistryError> {
        self.api_client.request_with_retry(
            Routes::CardList,
            RequestType::Get,
            None,
            query_params,
            headers,
        )
    }
}

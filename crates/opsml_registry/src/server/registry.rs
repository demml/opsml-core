#[cfg(feature = "server")]
pub mod server_logic {

    use opsml_error::error::RegistryError;
    use opsml_settings::config::OpsmlStorageSettings;
    use opsml_storage::StorageClientEnum;
    use opsml_types::*;

    pub struct ServerRegistry {
        registry_type: RegistryType,
        storage_client: StorageClientEnum,
    }

    impl ServerRegistry {
        pub async fn new(
            storage_settings: &mut OpsmlStorageSettings,
            registry_type: RegistryType,
        ) -> Result<Self, RegistryError> {
            let storage_client = StorageClientEnum::new(storage_settings)
                .await
                .map_err(|e| {
                    RegistryError::NewError(format!("Failed to create http client {}", e))
                })?;
            Ok(Self {
                registry_type,
                storage_client,
            })
        }

        pub fn table_name(&self) -> String {
            CardSQLTableNames::from_registry_type(&self.registry_type).to_string()
        }
    }
}

#[cfg(feature = "server")]
pub mod server_logic {

    use opsml_error::error::RegistryError;
    use opsml_settings::config::OpsmlConfig;
    use opsml_sql::{
        base::SqlClient,
        enums::client::{get_sql_client, SqlClientEnum},
        schemas::*,
    };
    use opsml_types::*;

    pub struct ServerRegistry {
        registry_type: RegistryType,
        sql_client: SqlClientEnum,
    }

    impl ServerRegistry {
        pub async fn new(
            config: &mut OpsmlConfig,
            registry_type: RegistryType,
        ) -> Result<Self, RegistryError> {
            let sql_client = get_sql_client(&config).await.map_err(|e| {
                RegistryError::NewError(format!("Failed to create sql client {}", e))
            })?;
            Ok(Self {
                registry_type,
                sql_client,
            })
        }

        pub fn table_name(&self) -> String {
            CardSQLTableNames::from_registry_type(&self.registry_type).to_string()
        }

        pub async fn list_cards(&mut self, args: CardQueryArgs) -> Result<Cards, RegistryError> {
            let table = CardSQLTableNames::from_registry_type(&self.registry_type);

            let cards = self
                .sql_client
                .query_cards(&table, &args)
                .await
                .map_err(|e| RegistryError::Error(format!("Failed to list cards {}", e)))?;

            match cards {
                CardResults::Data(data) => {
                    let cards = data
                        .into_iter()
                        .map(|card| convert_datacard(card))
                        .collect();
                    Ok(Cards::Data(cards))
                }
                CardResults::Model(data) => {
                    let cards = data
                        .into_iter()
                        .map(|card| convert_modelcard(card))
                        .collect();
                    Ok(Cards::Model(cards))
                }
                CardResults::Project(data) => {
                    let cards = data
                        .into_iter()
                        .map(|card| convert_projectcard(card))
                        .collect();
                    Ok(Cards::Project(cards))
                }
                CardResults::Run(data) => {
                    let cards = data.into_iter().map(|card| convert_runcard(card)).collect();
                    Ok(Cards::Run(cards))
                }
                CardResults::Pipeline(data) => {
                    let cards = data
                        .into_iter()
                        .map(|card| convert_pipelinecard(card))
                        .collect();
                    Ok(Cards::Pipeline(cards))
                }
                CardResults::Audit(data) => {
                    let cards = data
                        .into_iter()
                        .map(|card| convert_auditcard(card))
                        .collect();
                    Ok(Cards::Audit(cards))
                }
            }
        }
    }
}

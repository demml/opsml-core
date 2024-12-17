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
    use semver::Version;
    use sqlx::types::Json as SqlxJson;
    use tracing::error;

    #[derive(Debug)]
    pub struct ServerRegistry {
        sql_client: SqlClientEnum,
        pub table_name: CardSQLTableNames,
    }

    impl ServerRegistry {
        pub async fn new(
            config: &OpsmlConfig,
            registry_type: RegistryType,
        ) -> Result<Self, RegistryError> {
            let sql_client = get_sql_client(&config).await.map_err(|e| {
                RegistryError::NewError(format!("Failed to create sql client {}", e))
            })?;

            let table_name = CardSQLTableNames::from_registry_type(&registry_type);
            Ok(Self {
                sql_client,
                table_name,
            })
        }

        pub async fn list_cards(
            &mut self,
            args: CardQueryArgs,
        ) -> Result<Vec<Card>, RegistryError> {
            let cards = self
                .sql_client
                .query_cards(&self.table_name, &args)
                .await
                .map_err(|e| RegistryError::Error(format!("Failed to list cards {}", e)))?;

            match cards {
                CardResults::Data(data) => {
                    let cards = data
                        .into_iter()
                        .map(|card| convert_datacard(card))
                        .collect();
                    Ok(cards)
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

        pub async fn create_card(&self, card: &ClientCard) -> Result<(), RegistryError> {
            let card = match card.clone() {
                ClientCard::Data(client_card) => {
                    let server_card = DataCardRecord::new(
                        client_card.name,
                        client_card.repository,
                        client_card.version.parse().unwrap(),
                        client_card.contact,
                        client_card.tags,
                        client_card.data_type,
                        client_card.runcard_uid,
                        client_card.pipelinecard_uid,
                        client_card.auditcard_uid,
                        client_card.interface_type,
                    );
                    Card::Data(server_card)
                }
                ClientCard::Model(client_card) => {
                    let server_card = ModelCardRecord::new(
                        client_card.name,
                        client_card.repository,
                        client_card.version.parse().unwrap(),
                        client_card.contact,
                        client_card.tags,
                        client_card.datacard_uid,
                        client_card.sample_data_type,
                        client_card.model_type,
                        client_card.runcard_uid,
                        client_card.pipelinecard_uid,
                        client_card.auditcard_uid,
                        client_card.interface_type,
                        client_card.task_type,
                    );
                    Card::Model(server_card)
                }

                ClientCard::Project(client_card) => {
                    let server_card = ProjectCardRecord::new(
                        client_card.name,
                        client_card.repository,
                        client_card.version.parse().unwrap(),
                        client_card.project_id,
                    );
                    Card::Project(server_card)
                }

                ClientCard::Run(client_card) => {
                    let server_card = RunCardRecord::new(
                        client_card.name,
                        client_card.repository,
                        client_card.version.parse().unwrap(),
                        client_card.contact,
                        client_card.tags,
                        client_card.datacard_uids,
                        client_card.modelcard_uids,
                        client_card.pipelinecard_uid,
                        client_card.project,
                        client_card.artifact_uris,
                        client_card.compute_environment,
                    );
                    Card::Run(server_card)
                }

                ClientCard::Pipeline(client_card) => {
                    let server_card = PipelineCardRecord::new(
                        client_card.name,
                        client_card.repository,
                        client_card.version.parse().unwrap(),
                        client_card.contact,
                        client_card.tags,
                        client_card.pipeline_code_uri,
                        client_card.datacard_uids,
                        client_card.modelcard_uids,
                        client_card.runcard_uids,
                    );
                    Card::Pipeline(server_card)
                }

                ClientCard::Audit(client_card) => {
                    let server_card = AuditCardRecord::new(
                        client_card.name,
                        client_card.repository,
                        client_card.version.parse().unwrap(),
                        client_card.contact,
                        client_card.tags,
                        client_card.approved,
                        client_card.datacard_uids,
                        client_card.modelcard_uids,
                        client_card.runcard_uids,
                    );
                    Card::Audit(server_card)
                }
            };

            self.sql_client
                .insert_card(&self.table_name, &card)
                .await
                .map_err(|e| RegistryError::Error(format!("Failed to create card {}", e)))?;

            Ok(())
        }

        pub async fn update_card(&self, card: &ClientCard) -> Result<(), RegistryError> {
            let card = match card.clone() {
                ClientCard::Data(client_card) => {
                    let version = Version::parse(&client_card.version).map_err(|e| {
                        error!("Failed to parse version: {}", e);
                        RegistryError::Error("Failed to parse version".to_string())
                    })?;

                    let server_card = DataCardRecord {
                        uid: client_card.uid.unwrap(),
                        created_at: client_card.created_at,
                        app_env: client_card.app_env.unwrap(),
                        name: client_card.name,
                        repository: client_card.repository,
                        major: version.major as i32,
                        minor: version.minor as i32,
                        patch: version.patch as i32,
                        pre_tag: Some(version.pre.to_string()),
                        build_tag: Some(version.build.to_string()),
                        version: client_card.version,
                        contact: client_card.contact,
                        tags: SqlxJson(client_card.tags),
                        data_type: client_card.data_type,
                        runcard_uid: client_card.runcard_uid.unwrap(),
                        pipelinecard_uid: client_card.pipelinecard_uid.unwrap(),
                        auditcard_uid: client_card.auditcard_uid.unwrap(),
                        interface_type: client_card.interface_type.unwrap(),
                    };
                    Card::Data(server_card)
                }

                ClientCard::Model(client_card) => {
                    let version = Version::parse(&client_card.version).map_err(|e| {
                        error!("Failed to parse version: {}", e);
                        RegistryError::Error("Failed to parse version".to_string())
                    })?;

                    let server_card = ModelCardRecord {
                        uid: client_card.uid.unwrap(),
                        created_at: client_card.created_at,
                        app_env: client_card.app_env.unwrap(),
                        name: client_card.name,
                        repository: client_card.repository,
                        major: version.major as i32,
                        minor: version.minor as i32,
                        patch: version.patch as i32,
                        pre_tag: Some(version.pre.to_string()),
                        build_tag: Some(version.build.to_string()),
                        version: client_card.version,
                        contact: client_card.contact,
                        tags: SqlxJson(client_card.tags),
                        datacard_uid: client_card.datacard_uid.unwrap(),
                        sample_data_type: client_card.sample_data_type,
                        model_type: client_card.model_type,
                        runcard_uid: client_card.runcard_uid.unwrap(),
                        pipelinecard_uid: client_card.pipelinecard_uid.unwrap(),
                        auditcard_uid: client_card.auditcard_uid.unwrap(),
                        interface_type: client_card.interface_type.unwrap(),
                        task_type: client_card.task_type.unwrap(),
                    };
                    Card::Model(server_card)
                }

                ClientCard::Project(client_card) => {
                    let version = Version::parse(&client_card.version).map_err(|e| {
                        error!("Failed to parse version: {}", e);
                        RegistryError::Error("Failed to parse version".to_string())
                    })?;

                    let server_card = ProjectCardRecord {
                        uid: client_card.uid.unwrap(),
                        created_at: client_card.created_at,
                        name: client_card.name,
                        repository: client_card.repository,
                        major: version.major as i32,
                        minor: version.minor as i32,
                        patch: version.patch as i32,
                        pre_tag: Some(version.pre.to_string()),
                        build_tag: Some(version.build.to_string()),
                        version: client_card.version,
                        project_id: client_card.project_id,
                    };
                    Card::Project(server_card)
                }

                ClientCard::Run(client_card) => {
                    let version = Version::parse(&client_card.version).map_err(|e| {
                        error!("Failed to parse version: {}", e);
                        RegistryError::Error("Failed to parse version".to_string())
                    })?;

                    let server_card = RunCardRecord {
                        uid: client_card.uid.unwrap(),
                        created_at: client_card.created_at,
                        app_env: client_card.app_env.unwrap(),
                        name: client_card.name,
                        repository: client_card.repository,
                        major: version.major as i32,
                        minor: version.minor as i32,
                        patch: version.patch as i32,
                        pre_tag: Some(version.pre.to_string()),
                        build_tag: Some(version.build.to_string()),
                        version: client_card.version,
                        contact: client_card.contact,
                        tags: SqlxJson(client_card.tags),
                        datacard_uids: SqlxJson(client_card.datacard_uids.unwrap()),
                        modelcard_uids: SqlxJson(client_card.modelcard_uids.unwrap()),
                        pipelinecard_uid: client_card.pipelinecard_uid.unwrap(),
                        project: client_card.project,
                        artifact_uris: SqlxJson(client_card.artifact_uris.unwrap()),
                        compute_environment: SqlxJson(client_card.compute_environment.unwrap()),
                    };
                    Card::Run(server_card)
                }

                ClientCard::Pipeline(client_card) => {
                    let version = Version::parse(&client_card.version).map_err(|e| {
                        error!("Failed to parse version: {}", e);
                        RegistryError::Error("Failed to parse version".to_string())
                    })?;

                    let server_card = PipelineCardRecord {
                        uid: client_card.uid.unwrap(),
                        created_at: client_card.created_at,
                        app_env: client_card.app_env.unwrap(),
                        name: client_card.name,
                        repository: client_card.repository,
                        major: version.major as i32,
                        minor: version.minor as i32,
                        patch: version.patch as i32,
                        pre_tag: Some(version.pre.to_string()),
                        build_tag: Some(version.build.to_string()),
                        version: client_card.version,
                        contact: client_card.contact,
                        tags: SqlxJson(client_card.tags),
                        pipeline_code_uri: client_card.pipeline_code_uri,
                        datacard_uids: SqlxJson(client_card.datacard_uids.unwrap()),
                        modelcard_uids: SqlxJson(client_card.modelcard_uids.unwrap()),
                        runcard_uids: SqlxJson(client_card.runcard_uids.unwrap()),
                    };
                    Card::Pipeline(server_card)
                }

                ClientCard::Audit(client_card) => {
                    let version = Version::parse(&client_card.version).map_err(|e| {
                        error!("Failed to parse version: {}", e);
                        RegistryError::Error("Failed to parse version".to_string())
                    })?;

                    let server_card = AuditCardRecord {
                        uid: client_card.uid.unwrap(),
                        created_at: client_card.created_at,
                        app_env: client_card.app_env.unwrap(),
                        name: client_card.name,
                        repository: client_card.repository,
                        major: version.major as i32,
                        minor: version.minor as i32,
                        patch: version.patch as i32,
                        pre_tag: Some(version.pre.to_string()),
                        build_tag: Some(version.build.to_string()),
                        version: client_card.version,
                        contact: client_card.contact,
                        tags: SqlxJson(client_card.tags),
                        approved: client_card.approved,
                        datacard_uids: SqlxJson(client_card.datacard_uids.unwrap()),
                        modelcard_uids: SqlxJson(client_card.modelcard_uids.unwrap()),
                        runcard_uids: SqlxJson(client_card.runcard_uids.unwrap()),
                    };
                    Card::Audit(server_card)
                }
            };

            self.sql_client
                .update_card(&self.table_name, &card)
                .await
                .map_err(|e| RegistryError::Error(format!("Failed to update card {}", e)))?;

            Ok(())
        }

        pub async fn delete_card(&self, uid: &str) -> Result<(), RegistryError> {
            self.sql_client
                .delete_card(&self.table_name, uid)
                .await
                .map_err(|e| RegistryError::Error(format!("Failed to delete card {}", e)))?;

            Ok(())
        }
    }
}

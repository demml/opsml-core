use crate::cards::*;
use crate::enums::OpsmlRegistry;
use anyhow::{Context, Ok, Result as AnyhowResult};
use opsml_types::*;
use pyo3::prelude::*;
use std::collections::HashMap;

#[pyclass]
#[derive(Debug)]
pub struct CardRegistry {
    registry_type: RegistryType,
    table_name: String,
    registry: OpsmlRegistry,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl CardRegistry {
    #[new]
    pub fn new(registry_type: RegistryType) -> AnyhowResult<Self> {
        // Create a new tokio runtime for the registry (needed for async calls)
        let rt = tokio::runtime::Runtime::new().unwrap();

        let registry = rt
            .block_on(async { OpsmlRegistry::new(registry_type.clone()).await })
            .context("Failed to create registry")?;

        Ok(Self {
            registry_type: registry_type.clone(),
            table_name: CardSQLTableNames::from_registry_type(&registry_type).to_string(),
            registry,
            runtime: rt,
        })
    }

    #[getter]
    pub fn registry_type(&self) -> RegistryType {
        self.registry_type.clone()
    }

    #[getter]
    pub fn table_name(&self) -> &str {
        self.table_name.as_str()
    }

    pub fn mode(&self) -> RegistryMode {
        self.registry.mode()
    }

    #[pyo3(signature = (info=None, uid=None, name=None, repository=None, version=None, max_date=None, tags=None, limit=None, sort_by_timestamp=None))]
    pub fn list_cards(
        &mut self,
        info: Option<CardInfo>,
        uid: Option<String>,
        name: Option<String>,
        repository: Option<String>,
        version: Option<String>,
        max_date: Option<String>,
        tags: Option<HashMap<String, String>>,
        limit: Option<i32>,
        sort_by_timestamp: Option<bool>,
    ) -> AnyhowResult<Vec<Card>> {
        let mut uid = uid;
        let mut name = name;
        let mut repository = repository;
        let mut version = version;
        let mut tags = tags;

        if let Some(info) = info {
            name = name.or_else(|| info.name);
            repository = repository.or_else(|| info.repository);
            uid = uid.or_else(|| info.uid);
            version = version.or_else(|| info.version);
            tags = tags.or_else(|| info.tags);
        }

        if name.is_some() {
            name = Some(name.unwrap().to_lowercase());
        }

        if repository.is_some() {
            repository = Some(repository.unwrap().to_lowercase());
        }

        let limit_check = vec![
            uid.is_some(),
            name.is_some(),
            repository.is_some(),
            version.is_some(),
            tags.is_some(),
        ];

        // check if any value is true. If not, set limit to 25
        let limit = if limit_check.iter().any(|&x| x) {
            limit
        } else {
            Some(25)
        };

        let query_args = CardQueryArgs {
            uid,
            name,
            repository,
            version,
            max_date,
            tags,
            limit,
            sort_by_timestamp,
        };

        let cards = self
            .runtime
            .block_on(async { self.registry.list_cards(query_args).await })?;

        Ok(cards)
    }
}

#[pyclass]
#[derive(Debug)]
pub struct PyCardRegistry {
    registry_type: RegistryType,
    table_name: String,
    registry: OpsmlRegistry,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyCardRegistry {
    #[new]
    pub fn new(registry_type: RegistryType) -> AnyhowResult<Self> {
        // Create a new tokio runtime for the registry (needed for async calls)
        let rt = tokio::runtime::Runtime::new().unwrap();

        let registry = rt
            .block_on(async { OpsmlRegistry::new(registry_type.clone()).await })
            .context("Failed to create registry")?;

        Ok(Self {
            registry_type: registry_type.clone(),
            table_name: CardSQLTableNames::from_registry_type(&registry_type).to_string(),
            registry,
            runtime: rt,
        })
    }

    #[getter]
    pub fn registry_type(&self) -> RegistryType {
        self.registry_type.clone()
    }

    #[getter]
    pub fn table_name(&self) -> &str {
        &self.table_name
    }

    pub fn mode(&self) -> RegistryMode {
        self.registry.mode()
    }

    #[pyo3(signature = (info=None, uid=None, name=None, repository=None, version=None, max_date=None, tags=None, limit=None, sort_by_timestamp=None))]
    pub fn list_cards(
        &mut self,
        info: Option<CardInfo>,
        uid: Option<String>,
        name: Option<String>,
        repository: Option<String>,
        version: Option<String>,
        max_date: Option<String>,
        tags: Option<HashMap<String, String>>,
        limit: Option<i32>,
        sort_by_timestamp: Option<bool>,
    ) -> AnyhowResult<Vec<Card>> {
        let mut uid = uid;
        let mut name = name;
        let mut repository = repository;
        let mut version = version;
        let mut tags = tags;

        if let Some(info) = info {
            name = name.or_else(|| info.name);
            repository = repository.or_else(|| info.repository);
            uid = uid.or_else(|| info.uid);
            version = version.or_else(|| info.version);
            tags = tags.or_else(|| info.tags);
        }

        if name.is_some() {
            name = Some(name.unwrap().to_lowercase());
        }

        if repository.is_some() {
            repository = Some(repository.unwrap().to_lowercase());
        }

        let limit_check = vec![
            uid.is_some(),
            name.is_some(),
            repository.is_some(),
            version.is_some(),
            tags.is_some(),
        ];

        // check if any value is true. If not, set limit to 25
        let limit = if limit_check.iter().any(|&x| x) {
            limit
        } else {
            Some(25)
        };

        let query_args = CardQueryArgs {
            uid,
            name,
            repository,
            version,
            max_date,
            tags,
            limit,
            sort_by_timestamp,
        };

        let cards = self
            .runtime
            .block_on(async { self.registry.list_cards(query_args).await })?;

        Ok(cards)
    }

    pub fn register_card(&self, card: &Bound<'_, PyAny>) -> AnyhowResult<()> {
        if card.is_instance_of::<ModelCard>() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Card is not an instance of ModelCard"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opsml_settings::config::OpsmlDatabaseSettings;
    use opsml_sql::base::SqlClient;
    use opsml_sql::enums::client::SqlClientEnum;

    use std::env;

    fn cleanup() {
        // cleanup delete opsml.db and opsml_registries folder from the current directory
        let current_dir = std::env::current_dir().unwrap();
        // get 2 parents up
        let parent_dir = current_dir.parent().unwrap().parent().unwrap();
        let db_path = parent_dir.join("opsml.db");
        let registry_path = parent_dir.join("opsml_registries");

        if db_path.exists() {
            std::fs::remove_file(db_path).unwrap();
        }

        if registry_path.exists() {
            std::fs::remove_dir_all(registry_path).unwrap();
        }
    }

    fn create_registry_storage() {
        let current_dir = std::env::current_dir().unwrap();
        // get 2 parents up
        let parent_dir = current_dir.parent().unwrap().parent().unwrap();
        let registry_path = parent_dir.join("opsml_registries");

        // create the registry folder if it does not exist
        if !registry_path.exists() {
            std::fs::create_dir(registry_path).unwrap();
        }
    }

    fn get_connection_uri() -> String {
        let current_dir = env::current_dir().expect("Failed to get current directory");
        let parent_dir = current_dir.parent().unwrap().parent().unwrap();
        let db_path = parent_dir.join("opsml.db");

        format!(
            "sqlite://{}",
            db_path.to_str().expect("Failed to convert path to string")
        )
    }

    fn setup() {
        // create opsml_registries folder
        create_registry_storage();

        // create opsml.db and populate it with data
        let config = OpsmlDatabaseSettings {
            connection_uri: get_connection_uri(),
            max_connections: 1,
            sql_type: SqlType::Sqlite,
        };

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let client = SqlClientEnum::new(&config).await.unwrap();
            let script = std::fs::read_to_string("tests/populate_db.sql").unwrap();
            client.query(&script).await;
        });

        env::set_var("OPSML_TRACKING_URI", "http://0.0.0.0:3000");
    }

    #[test]
    fn test_registry_client_list_cards() {
        cleanup();

        //cleanup();
        setup();

        env::set_var("OPSML_TRACKING_URI", "http://0.0.0.0:3000");
        let mut registry = CardRegistry::new(RegistryType::Data).unwrap();

        // Test mode
        assert_eq!(registry.mode(), RegistryMode::Client);

        // Test table name
        assert_eq!(registry.table_name(), CardSQLTableNames::Data.to_string());

        // Test list cards
        let cards = registry
            .list_cards(None, None, None, None, None, None, None, None, None)
            .unwrap();

        assert_eq!(cards.len(), 10);

        cleanup();
    }
}

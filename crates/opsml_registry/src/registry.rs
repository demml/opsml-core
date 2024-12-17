use crate::enums::OpsmlRegistry;
use anyhow::{Context, Result as AnyhowResult};
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
    pub fn table_name(&self) -> String {
        self.table_name.clone()
    }

    #[pyo3(signature = (uid=None, name=None, repository=None, version=None, max_date=None, tags=None, limit=None, sort_by_timestamp=None))]
    pub fn list_cards(
        &mut self,
        uid: Option<String>,
        name: Option<String>,
        repository: Option<String>,
        version: Option<String>,
        max_date: Option<String>,
        tags: Option<HashMap<String, String>>,
        limit: Option<i32>,
        sort_by_timestamp: Option<bool>,
    ) -> AnyhowResult<Vec<Card>> {
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

    //pub fn list_cards(
    //    &mut self,
    //    uid: Option<String>,
    //    name: Option<String>,
    //    repository: Option<String>,
    //    version: Option<String>,
    //    max_date: Option<String>,
    //    tags: Option<HashMap<String, String>>,
    //    limit: Option<i32>,
    //    sort_by_timestamp: Option<bool>,
    //) -> PyResult<Vec<ClientCard>> {
    //    let query_args = CardQueryArgs {
    //        uid,
    //        name,
    //        repository,
    //        version,
    //        max_date,
    //        tags,
    //        limit,
    //        sort_by_timestamp,
    //    };
    //
    //    let cards = self.runtime.block_on(async {
    //        let cards = self.registry.list_cards(query_args).await?;
    //    })?;
    //
    //    cards
    //}
} //

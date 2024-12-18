use crate::cards::*;
use anyhow::{Context, Result as AnyhowResult};
use opsml_types::*;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct SerializedCard {
    name: String,
    repository: String,
    contact: String,
    version: String,
    uid: String,
    tags: HashMap<String, String>,
    metadata: DataCardMetadata,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DataCardMetadata {
    #[pyo3(get, set)]
    pub interface_type: String,

    #[pyo3(get, set)]
    pub data_type: String,

    #[pyo3(get, set)]
    pub description: Description,

    #[pyo3(get, set)]
    pub feature_map: HashMap<String, Feature>,

    #[pyo3(get, set)]
    pub runcard_uid: Option<String>,

    #[pyo3(get, set)]
    pub pipelinecard_uid: Option<String>,

    #[pyo3(get, set)]
    pub auditcard_uid: Option<String>,
}

#[pyclass]
#[derive(Debug)]
pub struct DataCard {
    #[pyo3(get)]
    pub interface: PyObject,

    #[pyo3(get)]
    pub name: String,

    #[pyo3(get)]
    pub repository: String,

    #[pyo3(get)]
    pub contact: String,

    #[pyo3(get)]
    pub version: String,

    #[pyo3(get)]
    pub uid: String,

    #[pyo3(get)]
    pub tags: HashMap<String, String>,

    #[pyo3(get)]
    pub metadata: DataCardMetadata,
}

#[pymethods]
impl DataCard {
    #[new]
    #[pyo3(signature = (interface, name=None, repository=None, contact=None, version=None, uid=None, info=None, tags=None, metadata=None))]
    pub fn new(
        interface: PyObject,
        name: Option<String>,
        repository: Option<String>,
        contact: Option<String>,
        version: Option<String>,
        uid: Option<String>,
        info: Option<CardInfo>,
        tags: Option<HashMap<String, String>>,
        metadata: Option<DataCardMetadata>,
    ) -> PyResult<Self> {
        let base_args = BaseArgs::new(
            name,
            repository,
            contact,
            version,
            uid,
            info,
            tags.unwrap_or_default(),
        )?;
        Ok(Self {
            interface,
            name: base_args.name,
            repository: base_args.repository,
            contact: base_args.contact,
            version: base_args.version,
            uid: base_args.uid,
            tags: base_args.tags,
            metadata: metadata.unwrap_or_default(),
        })
    }

    pub fn model_dump_json(&self) -> String {
        // serialize the struct to a string

        PyHelperFuncs::__json__(self.serialize_card())
    }

    #[staticmethod]
    pub fn model_validate_json(json_string: String, interface: PyObject) -> AnyhowResult<DataCard> {
        // deserialize the string to a struct
        let card: SerializedCard = serde_json::from_str(&json_string).with_context(|| {
            format!(
                "Failed to deserialize json string to card struct: {}",
                json_string
            )
        })?;

        Ok(DataCard {
            interface,
            name: card.name,
            repository: card.repository,
            contact: card.contact,
            version: card.version,
            uid: card.uid,
            tags: card.tags,
            metadata: card.metadata,
        })
    }
}

impl DataCard {
    fn serialize_card(&self) -> SerializedCard {
        SerializedCard {
            name: self.name.clone(),
            repository: self.repository.clone(),
            contact: self.contact.clone(),
            version: self.version.clone(),
            uid: self.uid.clone(),
            tags: self.tags.clone(),
            metadata: self.metadata.clone(),
        }
    }
}

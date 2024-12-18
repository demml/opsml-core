use crate::cards::*;
use anyhow::{Context, Result as AnyhowResult};
use opsml_error::error::CardError;
use opsml_types::*;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PySet};
use pyo3::{IntoPyObjectExt, PyObject};
use semver::Op;
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
    metadata: ModelCardMetadata,
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ModelCardMetadata {
    #[pyo3(get, set)]
    pub interface_type: String,

    #[pyo3(get, set)]
    pub description: Description,

    #[pyo3(get, set)]
    pub data_schema: DataSchema,

    #[pyo3(get, set)]
    pub runcard_uid: Option<String>,

    #[pyo3(get, set)]
    pub pipelinecard_uid: Option<String>,

    #[pyo3(get, set)]
    pub auditcard_uid: Option<String>,
}

#[pyclass]
#[derive(Debug)]
pub struct ModelCard {
    #[pyo3(get, set)]
    pub interface: PyObject,

    #[pyo3(get, set)]
    pub name: String,

    #[pyo3(get, set)]
    pub repository: String,

    #[pyo3(get, set)]
    pub contact: String,

    #[pyo3(get, set)]
    pub version: String,

    #[pyo3(get, set)]
    pub uid: String,

    #[pyo3(get, set)]
    pub tags: HashMap<String, String>,

    #[pyo3(get, set)]
    pub metadata: ModelCardMetadata,

    #[pyo3(get)]
    pub card_type: CardType,
}

#[pymethods]
impl ModelCard {
    #[new]
    #[pyo3(signature = (interface, name=None, repository=None, contact=None, version=None, uid=None, info=None, tags=None, metadata=None))]
    pub fn new(
        py: Python,
        interface: &Bound<'_, PyAny>,
        name: Option<String>,
        repository: Option<String>,
        contact: Option<String>,
        version: Option<String>,
        uid: Option<String>,
        info: Option<CardInfo>,
        tags: Option<HashMap<String, String>>,
        metadata: Option<ModelCardMetadata>,
    ) -> AnyhowResult<Self> {
        let base_args = BaseArgs::new(
            name,
            repository,
            contact,
            version,
            uid,
            info,
            tags.unwrap_or_default(),
        )?;

        // check if interface is a model interface (should be a bool)
        let is_interface: bool = interface
            .call_method0("is_model_interface")
            .with_context(|| "Error calling is_model_interface method on interface")?
            .extract()
            .unwrap();

        if !is_interface {
            return Err(CardError::Error("Interface is not a model interface".to_string()).into());
        }

        Ok(Self {
            interface: interface
                .into_py_any(py)
                .with_context(|| "Error converting interface to PyObject")?,
            name: base_args.name,
            repository: base_args.repository,
            contact: base_args.contact,
            version: base_args.version,
            uid: base_args.uid,
            tags: base_args.tags,
            metadata: metadata.unwrap_or_default(),
            card_type: CardType::Data,
        })
    }
}

impl ModelCard {
    pub fn serialize(&self) -> Result<(), CardError> {
        Python::with_gil(|py| {
            let obj = &self.interface;

            // Create the exclude dictionary
            let exclude_dict = PyDict::new(py);
            let exclude_set = PySet::new(
                py,
                &[
                    "model",
                    "preprocessor",
                    "sample_data",
                    "onnx_model",
                    "feature_extractor",
                    "tokenizer",
                    "drift_profile",
                ],
            )
            .map_err(|e| CardError::Error(e.to_string()))?;
            exclude_dict
                .set_item("exclude", exclude_set)
                .map_err(|e| CardError::Error(e.to_string()))?;

            // Call the model_dump method with the exclude argument
            let result = obj
                .call_method(py, "model_dump", (), Some(&exclude_dict))
                .map_err(|e| {
                    CardError::Error(format!(
                        "Error calling model_dump method on interface: {}",
                        e.to_string()
                    ))
                })?;

            // cast to pydict
            let dumped_interface = result
                .downcast_bound::<PyDict>(py)
                .map_err(|e| CardError::Error(e.to_string()))?;

            if let Ok(Some(onnx_args)) = dumped_interface.get_item("onnx_args") {
                let args = onnx_args
                    .downcast::<PyDict>()
                    .map_err(|e| CardError::Error(e.to_string()))?;

                // check if config in args. if it is, pop it
                if let Ok(Some(_)) = args.get_item("config") {
                    args.del_item("config")
                        .map_err(|e| CardError::Error(e.to_string()))?;
                }
            }

            // convert dumped interface to a string (json)
            let json = dumped_interface
                .to_string()
                .map_err(|e| CardError::Error(e.to_string()))?;

            println!("{:?}", result); // Print the result for debugging

            Ok(())
        })
    }
}

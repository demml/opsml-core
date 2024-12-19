use crate::shared::helper::PyHelperFuncs;
use opsml_error::error::TypeError;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use walkdir::WalkDir;

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum RegistryType {
    Data,
    Model,
    Run,
    Project,
    Audit,
    Pipeline,
    Metrics,
    HardwareMetrics,
    Parameters,
    Users,
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CardType {
    Data,
    Model,
    Run,
    Project,
    Audit,
    Pipeline,
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum CardSQLTableNames {
    Data,
    Model,
    Run,
    Project,
    Audit,
    Pipeline,
    Metrics,
    HardwareMetrics,
    Parameters,
    Users,
}

impl fmt::Display for CardSQLTableNames {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let table_name = match self {
            CardSQLTableNames::Data => "opsml_data_registry",
            CardSQLTableNames::Model => "opsml_model_registry",
            CardSQLTableNames::Run => "opsml_run_registry",
            CardSQLTableNames::Project => "opsml_project_registry",
            CardSQLTableNames::Audit => "opsml_audit_registry",
            CardSQLTableNames::Pipeline => "opsml_pipeline_registry",
            CardSQLTableNames::Metrics => "opsml_run_metrics",
            CardSQLTableNames::HardwareMetrics => "opsml_run_hardware_metrics",
            CardSQLTableNames::Parameters => "opsml_run_parameters",
            CardSQLTableNames::Users => "opsml_users",
        };
        write!(f, "{}", table_name)
    }
}

impl CardSQLTableNames {
    pub fn from_registry_type(registry_type: &RegistryType) -> Self {
        match registry_type {
            RegistryType::Data => CardSQLTableNames::Data,
            RegistryType::Model => CardSQLTableNames::Model,
            RegistryType::Run => CardSQLTableNames::Run,
            RegistryType::Project => CardSQLTableNames::Project,
            RegistryType::Audit => CardSQLTableNames::Audit,
            RegistryType::Pipeline => CardSQLTableNames::Pipeline,
            RegistryType::Metrics => CardSQLTableNames::Metrics,
            RegistryType::HardwareMetrics => CardSQLTableNames::HardwareMetrics,
            RegistryType::Parameters => CardSQLTableNames::Parameters,
            RegistryType::Users => CardSQLTableNames::Users,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub enum VersionType {
    Major,
    Minor,
    Patch,
    Pre,
    Build,
    PreBuild,
}

impl FromStr for VersionType {
    type Err = ();

    fn from_str(input: &str) -> Result<VersionType, Self::Err> {
        match input.to_lowercase().as_str() {
            "major" => Ok(VersionType::Major),
            "minor" => Ok(VersionType::Minor),
            "patch" => Ok(VersionType::Patch),
            "pre" => Ok(VersionType::Pre),
            "build" => Ok(VersionType::Build),
            "pre_build" => Ok(VersionType::PreBuild),
            _ => Err(()),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Description {
    #[pyo3(get, set)]
    pub summary: Option<String>,

    #[pyo3(get, set)]
    pub sample_code: Option<String>,

    #[pyo3(get, set)]
    pub notes: Option<String>,
}

#[pymethods]
impl Description {
    #[new]
    #[pyo3(signature = (summary=None, sample_code=None, notes=None))]
    fn new(
        summary: Option<String>,
        sample_code: Option<String>,
        notes: Option<String>,
    ) -> Result<Self, TypeError> {
        Ok(Description {
            summary: Description::find_filepath(summary)?,
            sample_code,
            notes,
        })
    }
    pub fn __str__(&self) -> String {
        // serialize the struct to a string
        PyHelperFuncs::__str__(self)
    }
}

impl Description {
    pub fn find_filepath(filepath: Option<String>) -> Result<Option<String>, TypeError> {
        match filepath {
            Some(path) => {
                let current_dir = std::env::current_dir().map_err(|_| TypeError::FileEntryError)?;
                // recursively search for file in current directory
                for entry in WalkDir::new(current_dir) {
                    let entry = entry.map_err(|_| TypeError::FileEntryError)?;
                    if entry.file_type().is_file() && entry.file_name().to_string_lossy() == path {
                        return Ok(Some(entry.path().to_str().unwrap().to_string()));
                    }
                }
                // raise error if file not found
                Err(TypeError::FileNotFoundError)
            }
            None => Ok(None),
        }
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Feature {
    #[pyo3(get, set)]
    feature_type: String,
    #[pyo3(get, set)]
    shape: Vec<i32>,
    #[pyo3(get, set)]
    extra_args: HashMap<String, String>,
}

#[pymethods]
impl Feature {
    #[new]
    #[pyo3(signature = (feature_type, shape, extra_args=None))]
    fn new(
        feature_type: String,
        shape: Vec<i32>,
        extra_args: Option<HashMap<String, String>>,
    ) -> Self {
        Feature {
            feature_type,
            shape,
            extra_args: extra_args.unwrap_or_default(),
        }
    }

    pub fn __str__(&self) -> String {
        // serialize the struct to a string
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct OnnxSchema {
    #[pyo3(get, set)]
    pub input_features: HashMap<String, Feature>,

    #[pyo3(get, set)]
    pub output_features: HashMap<String, Feature>,

    #[pyo3(get, set)]
    pub onnx_version: String,
}

#[pymethods]
impl OnnxSchema {
    #[new]
    #[pyo3(signature = (input_features, output_features, onnx_version))]
    fn new(
        input_features: HashMap<String, Feature>,
        output_features: HashMap<String, Feature>,
        onnx_version: String,
    ) -> Self {
        OnnxSchema {
            input_features,
            output_features,
            onnx_version,
        }
    }
    pub fn __str__(&self) -> String {
        // serialize the struct to a string
        PyHelperFuncs::__str__(self)
    }
}

#[pyclass]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct DataSchema {
    #[pyo3(get, set)]
    pub data_type: String,

    #[pyo3(get, set)]
    pub input_features: Option<HashMap<String, Feature>>,

    #[pyo3(get, set)]
    pub output_features: Option<HashMap<String, Feature>>,

    #[pyo3(get, set)]
    pub onnx_schema: Option<OnnxSchema>,
}

#[pymethods]
impl DataSchema {
    #[new]
    #[pyo3(signature = (data_type, input_features=None, output_features=None, onnx_schema=None))]
    fn new(
        data_type: String,
        input_features: Option<HashMap<String, Feature>>,
        output_features: Option<HashMap<String, Feature>>,
        onnx_schema: Option<OnnxSchema>,
    ) -> Self {
        DataSchema {
            data_type,
            input_features,
            output_features,
            onnx_schema,
        }
    }
    pub fn __str__(&self) -> String {
        // serialize the struct to a string
        PyHelperFuncs::__str__(self)
    }
}

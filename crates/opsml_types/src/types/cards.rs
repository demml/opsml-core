use crate::helper::PyHelperFuncs;
use opsml_error::error::TypeError;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use walkdir::WalkDir;

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
                    if entry.file_type().is_file() {
                        if entry.file_name().to_string_lossy() == path {
                            return Ok(Some(entry.path().to_str().unwrap().to_string()));
                        }
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

// new should implement form a pytuple

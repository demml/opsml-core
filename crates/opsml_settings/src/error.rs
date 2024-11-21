use pyo3::PyErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("Failure: {0}")]
    Error(String),
}

impl From<SettingsError> for PyErr {
    fn from(err: SettingsError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.to_string())
    }
}

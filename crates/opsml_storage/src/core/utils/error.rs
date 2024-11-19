use pyo3::PyErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Api client failure: {0}")]
    Error(String),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage client failure: {0}")]
    Error(String),

    #[error("Storage client failure. Unsupported client.")]
    UnsupportedClient,

    #[error("Storage client failure. Unsupported operation.")]
    UnsupportedOperation,
}

impl From<StorageError> for PyErr {
    fn from(err: StorageError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(err.to_string())
    }
}

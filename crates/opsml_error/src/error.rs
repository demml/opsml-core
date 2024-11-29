use pyo3::PyErr;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SettingsError {
    #[error("Settings Error: {0}")]
    Error(String),
}

impl From<SettingsError> for PyErr {
    fn from(err: SettingsError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(err.to_string())
    }
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage Error: {0}")]
    Error(String),
}

impl From<StorageError> for PyErr {
    fn from(err: StorageError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(err.to_string())
    }
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Api Error: {0}")]
    Error(String),
}

impl From<ApiError> for PyErr {
    fn from(err: ApiError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(err.to_string())
    }
}

#[derive(Error, Debug)]
pub enum UtilError {
    #[error("Util Error: {0}")]
    Error(String),

    #[error("Error serializing data")]
    SerializationError,

    #[error("Error creating path")]
    CreatePathError,

    #[error("Error getting parent path")]
    GetParentPathError,

    #[error("Failed to create directory")]
    CreateDirectoryError,

    #[error("Failed to write to file")]
    WriteError,
}

impl From<UtilError> for PyErr {
    fn from(err: UtilError) -> PyErr {
        PyErr::new::<pyo3::exceptions::PyValueError, _>(err.to_string())
    }
}

#[derive(Error, Debug)]
pub enum LoggingError {
    #[error("Logging Error: {0}")]
    Error(String),
}

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ServerError {
    #[error("Failed to delete file: {0}")]
    DeleteError(String),

    #[error("Failed to create multipart: {0}")]
    MultipartError(String),

    #[error("Failed to presign: {0}")]
    PresignedError(String),

    #[error("Failed to list files: {0}")]
    ListFileError(String),
}

#[derive(Error, Debug)]
pub enum SqlError {
    #[error("Failed to run sql migrations: {0}")]
    MigrationError(String),
}

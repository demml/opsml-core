/// Implements a generic enum to handle different storage clients based on the storage URI
/// This enum is meant to provide a common interface to use in the server
use crate::core::storage::base::{FileInfo, FileSystem, StorageSettings};
use crate::core::storage::local::LocalFSStorageClient;
use crate::core::utils::error::StorageError;
use pyo3::prelude::*;
use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "google_storage")]
use crate::core::storage::google::google_storage::GCSFSStorageClient;

#[cfg(feature = "aws_storage")]
use crate::core::storage::aws::aws_storage::S3FStorageClient;

pub enum StorageClientEnum {
    #[cfg(feature = "google_storage")]
    Google(GCSFSStorageClient),
    #[cfg(feature = "aws_storage")]
    AWS(S3FStorageClient),
    Local(LocalFSStorageClient),
}

impl StorageClientEnum {
    pub fn name(&self) -> &str {
        match self {
            #[cfg(feature = "google_storage")]
            StorageClientEnum::Google(client) => client.name(),
            #[cfg(feature = "aws_storage")]
            StorageClientEnum::AWS(client) => client.name(),
            StorageClientEnum::Local(client) => client.name(),
        }
    }
    // Implement the required methods for the StorageClient trait
    // For example:
    pub async fn new(settings: StorageSettings) -> Result<Self, StorageError> {
        // match start of storage uri with starts_with("gs://") or starts_with("s3://")
        // to determine the storage type

        match settings.storage_uri {
            #[cfg(feature = "google_storage")]
            _ if settings.storage_uri.starts_with("gs://") => {
                // strip the gs:// prefix
                let client = GCSFSStorageClient::new(settings).await;
                Ok(StorageClientEnum::Google(client))
            }
            #[cfg(feature = "aws_storage")]
            _ if settings.storage_uri.starts_with("s3://") => {
                // strip the s3:// prefix
                let client = S3FStorageClient::new(settings).await;
                Ok(StorageClientEnum::AWS(client))
            }
            _ => {
                let client = LocalFSStorageClient::new(settings).await;
                Ok(StorageClientEnum::Local(client))
            }
        }
    }

    pub async fn find(&self, path: &Path) -> Result<Vec<String>, StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            StorageClientEnum::Google(client) => client.find(path).await,
            #[cfg(feature = "aws_storage")]
            StorageClientEnum::AWS(client) => client.find(path).await,
            StorageClientEnum::Local(client) => client.find(path).await,
        }
    }

    pub async fn find_info(&self, path: &Path) -> Result<Vec<FileInfo>, StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            StorageClientEnum::Google(client) => client.find_info(path).await,
            #[cfg(feature = "aws_storage")]
            StorageClientEnum::AWS(client) => client.find_info(path).await,
            StorageClientEnum::Local(client) => client.find_info(path).await,
        }
    }

    pub async fn get(
        &self,
        lpath: &Path,
        rpath: &Path,
        recursive: bool,
    ) -> Result<(), StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            StorageClientEnum::Google(client) => client.get(lpath, rpath, recursive).await,
            #[cfg(feature = "aws_storage")]
            StorageClientEnum::AWS(client) => client.get(lpath, rpath, recursive).await,
            StorageClientEnum::Local(client) => client.get(lpath, rpath, recursive).await,
        }
    }

    pub async fn put(
        &self,
        lpath: &Path,
        rpath: &Path,
        recursive: bool,
    ) -> Result<(), StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            StorageClientEnum::Google(client) => client.put(lpath, rpath, recursive).await,
            #[cfg(feature = "aws_storage")]
            StorageClientEnum::AWS(client) => client.put(lpath, rpath, recursive).await,
            StorageClientEnum::Local(client) => client.put(lpath, rpath, recursive).await,
        }
    }

    pub async fn copy(&self, src: &Path, dest: &Path, recursive: bool) -> Result<(), StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            StorageClientEnum::Google(client) => client.copy(src, dest, recursive).await,
            #[cfg(feature = "aws_storage")]
            StorageClientEnum::AWS(client) => client.copy(src, dest, recursive).await,
            StorageClientEnum::Local(client) => client.copy(src, dest, recursive).await,
        }
    }

    pub async fn rm(&self, path: &Path, recursive: bool) -> Result<(), StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            StorageClientEnum::Google(client) => client.rm(path, recursive).await,
            #[cfg(feature = "aws_storage")]
            StorageClientEnum::AWS(client) => client.rm(path, recursive).await,
            StorageClientEnum::Local(client) => client.rm(path, recursive).await,
        }
    }

    pub async fn exists(&self, path: &Path) -> Result<bool, StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            StorageClientEnum::Google(client) => client.exists(path).await,
            #[cfg(feature = "aws_storage")]
            StorageClientEnum::AWS(client) => client.exists(path).await,
            StorageClientEnum::Local(client) => client.exists(path).await,
        }
    }

    pub async fn generate_presigned_url(
        &self,
        path: &Path,
        expiration: u64,
    ) -> Result<String, StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            StorageClientEnum::Google(client) => {
                client.generate_presigned_url(path, expiration).await
            }
            #[cfg(feature = "aws_storage")]
            StorageClientEnum::AWS(client) => client.generate_presigned_url(path, expiration).await,
            StorageClientEnum::Local(client) => {
                client.generate_presigned_url(path, expiration).await
            }
        }
    }

    pub async fn create_multipart_upload(&self, path: &Path) -> Result<String, StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            StorageClientEnum::Google(client) => client.create_multipart_upload(path).await,
            #[cfg(feature = "aws_storage")]
            StorageClientEnum::AWS(client) => client.create_multipart_upload(path).await,
            StorageClientEnum::Local(client) => client.create_multipart_upload(path).await,
        }
    }
}

#[pyclass]
pub struct PyStorageClient {
    inner: StorageClientEnum,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyStorageClient {
    #[new]
    fn new(settings: StorageSettings) -> PyResult<Self> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = rt
            .block_on(StorageClientEnum::new(settings))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;
        Ok(PyStorageClient {
            inner: client,
            runtime: rt,
        })
    }

    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    #[pyo3(signature = (path=PathBuf::new()))]
    fn find(&self, path: PathBuf) -> PyResult<Vec<String>> {
        let result = self
            .runtime
            .block_on(self.inner.find(&path))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;
        Ok(result)
    }

    fn find_info(&self, path: PathBuf) -> PyResult<Vec<FileInfo>> {
        let result = self
            .runtime
            .block_on(self.inner.find_info(&path))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;
        Ok(result)
    }

    #[pyo3(signature = (lpath, rpath, recursive = false))]
    pub fn get(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.get(&lpath, &rpath, recursive))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;
        Ok(())
    }

    #[pyo3(signature = (lpath, rpath, recursive = false))]
    pub fn put(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.put(&lpath, &rpath, recursive))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;
        Ok(())
    }

    pub fn copy(&self, src: PathBuf, dest: PathBuf, recursive: bool) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.copy(&src, &dest, recursive))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;
        Ok(())
    }

    pub fn rm(&self, path: PathBuf, recursive: bool) -> PyResult<()> {
        self.runtime
            .block_on(self.inner.rm(&path, recursive))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;

        Ok(())
    }

    pub fn exists(&self, path: PathBuf) -> PyResult<bool> {
        let result = self
            .runtime
            .block_on(self.inner.exists(&path))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;
        Ok(result)
    }

    pub fn generate_presigned_url(&self, path: PathBuf, expiration: u64) -> PyResult<String> {
        let result = self
            .runtime
            .block_on(self.inner.generate_presigned_url(&path, expiration))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{:?}", e)))?;
        Ok(result)
    }
}

#[pyfunction]
pub fn get_storage_client(settings: StorageSettings) -> PyResult<PyStorageClient> {
    PyStorageClient::new(settings)
}

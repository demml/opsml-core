/// Implements a generic enum to handle different storage clients based on the storage URI
/// This enum is meant to provide a common interface to use in the server
use crate::storage::base::FileSystem;
use crate::storage::local::client::{LocalFSStorageClient, LocalMultiPartUpload};

use anyhow::Context;
use anyhow::Result as AnyhowResult;
use opsml_contracts::FileInfo;
use opsml_contracts::UploadPartArgs;
use opsml_error::error::StorageError;
use opsml_settings::config::{OpsmlConfig, OpsmlStorageSettings, StorageType};
use pyo3::prelude::*;
use std::path::Path;
use std::path::PathBuf;

use crate::storage::aws::client::{AWSMulitPartUpload, S3FStorageClient};
use crate::storage::gcs::client::{GCSFSStorageClient, GoogleMultipartUpload};

pub enum MultiPartUploader {
    Google(GoogleMultipartUpload),
    AWS(AWSMulitPartUpload),
    Local(LocalMultiPartUpload),
}

impl MultiPartUploader {
    pub fn session_url(&self) -> String {
        match self {
            MultiPartUploader::Google(uploader) => uploader.upload_client.url().to_string().clone(),
            MultiPartUploader::AWS(uploader) => uploader.upload_id.clone(),
            MultiPartUploader::Local(uploader) => {
                uploader.rpath.clone().to_str().unwrap().to_string()
            }
        }
    }

    pub async fn complete_upload(&mut self) -> Result<(), StorageError> {
        match self {
            MultiPartUploader::Google(uploader) => uploader.complete_upload().await,

            MultiPartUploader::AWS(uploader) => uploader.complete_upload().await,
            MultiPartUploader::Local(uploader) => uploader.complete_upload().await,
        }
    }
}

pub enum StorageClientEnum {
    Google(GCSFSStorageClient),
    AWS(S3FStorageClient),
    Local(LocalFSStorageClient),
}

impl StorageClientEnum {
    pub fn name(&self) -> &str {
        match self {
            StorageClientEnum::Google(client) => client.name(),

            StorageClientEnum::AWS(client) => client.name(),
            StorageClientEnum::Local(client) => client.name(),
        }
    }

    pub fn storage_type(&self) -> StorageType {
        match self {
            StorageClientEnum::Google(_) => StorageType::Google,

            StorageClientEnum::AWS(_) => StorageType::AWS,
            StorageClientEnum::Local(_) => StorageType::Local,
        }
    }

    pub async fn new(settings: &OpsmlStorageSettings) -> Result<Self, StorageError> {
        match settings.storage_type {
            StorageType::Google => {
                // strip the gs:// prefix
                let client = GCSFSStorageClient::new(settings).await;
                Ok(StorageClientEnum::Google(client))
            }

            StorageType::AWS => {
                // strip the s3:// prefix
                let client = S3FStorageClient::new(settings).await;
                Ok(StorageClientEnum::AWS(client))
            }
            StorageType::Local => {
                let client = LocalFSStorageClient::new(settings).await;
                Ok(StorageClientEnum::Local(client))
            }
        }
    }

    pub async fn find(&self, path: &Path) -> Result<Vec<String>, StorageError> {
        match self {
            StorageClientEnum::Google(client) => client.find(path).await,

            StorageClientEnum::AWS(client) => client.find(path).await,
            StorageClientEnum::Local(client) => client.find(path).await,
        }
    }

    pub async fn find_info(&self, path: &Path) -> Result<Vec<FileInfo>, StorageError> {
        match self {
            StorageClientEnum::Google(client) => client.find_info(path).await,
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
            StorageClientEnum::Google(client) => client.get(lpath, rpath, recursive).await,

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
            StorageClientEnum::Google(client) => client.put(lpath, rpath, recursive).await,

            StorageClientEnum::AWS(client) => client.put(lpath, rpath, recursive).await,
            StorageClientEnum::Local(client) => client.put(lpath, rpath, recursive).await,
        }
    }

    pub async fn copy(&self, src: &Path, dest: &Path, recursive: bool) -> Result<(), StorageError> {
        match self {
            StorageClientEnum::Google(client) => client.copy(src, dest, recursive).await,

            StorageClientEnum::AWS(client) => client.copy(src, dest, recursive).await,
            StorageClientEnum::Local(client) => client.copy(src, dest, recursive).await,
        }
    }

    pub async fn rm(&self, path: &Path, recursive: bool) -> Result<(), StorageError> {
        match self {
            StorageClientEnum::Google(client) => client.rm(path, recursive).await,

            StorageClientEnum::AWS(client) => client.rm(path, recursive).await,
            StorageClientEnum::Local(client) => client.rm(path, recursive).await,
        }
    }

    pub async fn exists(&self, path: &Path) -> Result<bool, StorageError> {
        match self {
            StorageClientEnum::Google(client) => client.exists(path).await,

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
            StorageClientEnum::Google(client) => {
                client.generate_presigned_url(path, expiration).await
            }

            StorageClientEnum::AWS(client) => client.generate_presigned_url(path, expiration).await,
            StorageClientEnum::Local(client) => {
                client.generate_presigned_url(path, expiration).await
            }
        }
    }

    pub async fn generate_presigned_url_for_part(
        &self,
        part_number: i32,
        path: &str,
        session_url: String,
    ) -> Result<String, StorageError> {
        match self {
            StorageClientEnum::Google(_client) => Ok(session_url),

            StorageClientEnum::AWS(client) => {
                client
                    .generate_presigned_url_for_part(part_number, path, &session_url)
                    .await
            }
            StorageClientEnum::Local(_client) => Ok(session_url),
        }
    }

    pub async fn create_multipart_upload(&self, path: &Path) -> Result<String, StorageError> {
        match self {
            StorageClientEnum::Google(client) => {
                // google returns the session uri
                let result = client
                    .client()
                    .create_multipart_upload(path.to_str().unwrap())
                    .await?;

                Ok(result.url().to_string())
            }

            StorageClientEnum::AWS(client) => {
                // aws returns the session uri
                client
                    .client()
                    .create_multipart_upload(path.to_str().unwrap())
                    .await
            }
            StorageClientEnum::Local(client) => {
                // local returns the path
                client
                    .client()
                    .create_multipart_upload(path.to_str().unwrap())
                    .await
            }
        }
    }

    pub async fn create_multipart_uploader(
        &self,
        lpath: &Path,
        rpath: &Path,
        session_url: String,
    ) -> Result<MultiPartUploader, StorageError> {
        match self {
            StorageClientEnum::Google(client) => {
                let uploader = client
                    .create_multipart_uploader(lpath, rpath, Some(session_url))
                    .await?;
                Ok(MultiPartUploader::Google(uploader))
            }

            StorageClientEnum::AWS(client) => {
                let uploader = client
                    .create_multipart_uploader(rpath, lpath, Some(session_url))
                    .await?;
                Ok(MultiPartUploader::AWS(uploader))
            }
            StorageClientEnum::Local(client) => LocalMultiPartUpload::new(rpath, session_url)
                .await
                .map(|uploader| MultiPartUploader::Local(uploader)),
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
    pub fn new(settings: &OpsmlStorageSettings) -> Result<Self, StorageError> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = rt
            .block_on(StorageClientEnum::new(settings))
            .map_err(|e| StorageError::Error(format!("{:?}", e)))?;

        Ok(PyStorageClient {
            inner: client,
            runtime: rt,
        })
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

pub async fn get_storage_system(config: &OpsmlConfig) -> AnyhowResult<StorageClientEnum> {
    // check storage_uri for prefix
    let storage_settings = config.storage_settings();

    StorageClientEnum::new(&storage_settings)
        .await
        .with_context(|| {
            format!(
                "Failed to create storage client for storage type: {:?}",
                storage_settings.storage_type
            )
        })
}

#[pyfunction]
pub fn get_opsml_storage_system(config: &OpsmlConfig) -> AnyhowResult<PyStorageClient> {
    // check storage_uri for prefix
    let storage_settings = config.storage_settings();

    PyStorageClient::new(&storage_settings).with_context(|| {
        format!(
            "Failed to create storage client for storage type: {:?}",
            storage_settings.storage_type
        )
    })
}
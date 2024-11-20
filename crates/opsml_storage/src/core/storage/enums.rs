use crate::core::http_client::client::HttpMultiPartUpload;
/// Implements a generic enum to handle different storage clients based on the storage URI
/// This enum is meant to provide a common interface to use in the server
use crate::core::storage::base::{FileInfo, FileSystem, StorageSettings};
use crate::core::storage::local::{LocalFSStorageClient, LocalMultiPartUpload};
use crate::core::utils::error::StorageError;
use aws_smithy_types::byte_stream::ByteStream;
use pyo3::prelude::*;
use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "google_storage")]
use crate::core::storage::google::google_storage::{GCSFSStorageClient, GoogleMultipartUpload};

#[cfg(feature = "aws_storage")]
use crate::core::storage::aws::aws_storage::{AWSMulitPartUpload, S3FStorageClient};

pub enum MultiPartUploader {
    #[cfg(feature = "google_storage")]
    Google(GoogleMultipartUpload),
    #[cfg(feature = "aws_storage")]
    AWS(AWSMulitPartUpload),
    Local(LocalMultiPartUpload),
    HTTP(HttpMultiPartUpload),
}

impl MultiPartUploader {
    pub async fn new(
        storage_type: StorageType,
        settings: StorageSettings,
        path: &Path,
    ) -> Result<Self, StorageError> {
        match storage_type {
            #[cfg(feature = "google_storage")]
            StorageType::Google => {
                let client = GCSFSStorageClient::new(settings).await;
                let uploader = GoogleMultipartUpload::new(
                    &client.client().client,
                    &client.client().bucket,
                    path.to_str().unwrap(),
                )
                .await
                .map_err(|e| StorageError::Error(format!("{:?}", e)))?;
                Ok(MultiPartUploader::Google(uploader))
            }
            #[cfg(feature = "aws_storage")]
            StorageType::AWS => {
                let client = S3FStorageClient::new(settings).await;
                let uploader =
                    AWSMulitPartUpload::new(&client.client().bucket, path.to_str().unwrap())
                        .await
                        .map_err(|e| StorageError::Error(format!("{:?}", e)))?;

                Ok(MultiPartUploader::AWS(uploader))
            }
            StorageType::Local => {
                unimplemented!("Local storage is not supported for multipart uploads")
            }
        }
    }
    pub async fn upload_part(
        &mut self,
        first_byte: &u64,
        last_byte: &u64,
        part_number: &i32,
        total_size: &u64,
        body: bytes::Bytes,
    ) -> Result<bool, StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            MultiPartUploader::Google(uploader) => {
                let stream = ByteStream::from(body);
                uploader
                    .upload_part(stream, first_byte, last_byte, total_size)
                    .await?;
                Ok(true)
            }
            #[cfg(feature = "aws_storage")]
            MultiPartUploader::AWS(uploader) => {
                let stream = ByteStream::from(body);
                uploader.upload_part(*part_number, stream).await
            }

            MultiPartUploader::Local(uploader) => uploader.upload_part(body).await,
            MultiPartUploader::HTTP(_uploader) => {
                // this should only raise an error

                Ok(false)
            }
        }
    }

    pub async fn get_next_chunk(
        &self,
        path: &Path,
        chunk_size: u64,
        chunk_index: u64,
        this_chunk_size: u64,
    ) -> Result<ByteStream, StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            MultiPartUploader::Google(uploader) => {
                uploader
                    .get_next_chunk(path, chunk_size, chunk_index, this_chunk_size)
                    .await
            }
            #[cfg(feature = "aws_storage")]
            MultiPartUploader::AWS(uploader) => {
                uploader
                    .get_next_chunk(path, chunk_size, chunk_index, this_chunk_size)
                    .await
            }
            MultiPartUploader::Local(uploader) => {
                uploader
                    .get_next_chunk(path, chunk_size, chunk_index, this_chunk_size)
                    .await
            }
            MultiPartUploader::HTTP(_uploader) => {
                // this should only raise an error
                unimplemented!()
            }
        }
    }

    pub async fn complete_upload(&mut self) -> Result<(), StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            MultiPartUploader::Google(uploader) => uploader.complete_upload().await,
            #[cfg(feature = "aws_storage")]
            MultiPartUploader::AWS(uploader) => uploader.complete_upload().await,
            MultiPartUploader::Local(uploader) => uploader.complete_upload().await,
            MultiPartUploader::HTTP(_uploader) => {
                // this should only raise an error
                Ok(())
            }
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Debug, PartialEq, Clone)]
pub enum StorageType {
    Google,
    AWS,
    Local,
}

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

    pub async fn new(
        storage_type: StorageType,
        settings: StorageSettings,
    ) -> Result<Self, StorageError> {
        match storage_type {
            #[cfg(feature = "google_storage")]
            StorageType::Google => {
                // strip the gs:// prefix
                let client = GCSFSStorageClient::new(settings).await;
                Ok(StorageClientEnum::Google(client))
            }
            #[cfg(feature = "aws_storage")]
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
            StorageClientEnum::Google(client) => {
                // google returns the session uri
                client
                    .client()
                    .create_multipart_upload(path.to_str().unwrap())
                    .await
            }
            #[cfg(feature = "aws_storage")]
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
}

#[pyclass]
pub struct PyStorageClient {
    inner: StorageClientEnum,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyStorageClient {
    #[new]
    fn new(storage_type: StorageType, settings: StorageSettings) -> PyResult<Self> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let client = rt
            .block_on(StorageClientEnum::new(storage_type, settings))
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
pub fn get_storage_client(
    storage_type: StorageType,
    settings: StorageSettings,
) -> PyResult<PyStorageClient> {
    PyStorageClient::new(storage_type, settings)
}

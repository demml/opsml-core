use crate::storage::base::get_files;
use crate::storage::base::PathExt;
use crate::storage::http::base::{build_http_client, HttpStorageClient};
use anyhow::Context;
use anyhow::Result as AnyhowResult;
use opsml_contracts::FileInfo;
use opsml_error::error::StorageError;
use opsml_settings::config::OpsmlStorageSettings;
use opsml_settings::config::StorageType;
use opsml_utils::color::LogColors;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

pub struct HttpFSStorageClient {
    client: HttpStorageClient,
}

impl HttpFSStorageClient {
    pub fn storage_type(&self) -> StorageType {
        self.client.storage_type.clone()
    }
    pub fn name(&self) -> &str {
        "HttpFSStorageClient"
    }

    pub async fn new(settings: &mut OpsmlStorageSettings) -> Result<Self, StorageError> {
        let client = build_http_client(&settings.api_settings)
            .map_err(|e| StorageError::Error(format!("Failed to create http client {}", e)))?;

        Ok(HttpFSStorageClient {
            client: HttpStorageClient::new(settings, &client).await.unwrap(),
        })
    }

    pub async fn find(&mut self, path: &Path) -> Result<Vec<String>, StorageError> {
        self.client.find(path.to_str().unwrap()).await
    }

    pub async fn find_info(&mut self, path: &Path) -> Result<Vec<FileInfo>, StorageError> {
        self.client.find_info(path.to_str().unwrap()).await
    }

    pub async fn get(
        &mut self,
        lpath: &Path,
        rpath: &Path,
        recursive: bool,
    ) -> Result<(), StorageError> {
        if recursive {
            // list all objects in the path
            let objects = self.client.find(rpath.to_str().unwrap()).await?;

            // iterate over each object and get it
            for obj in objects {
                let file_path = Path::new(obj.as_str());
                let relative_path = file_path.relative_path(rpath)?;
                let local_path = lpath.join(relative_path);

                self.client
                    .get_object(local_path.to_str().unwrap(), file_path.to_str().unwrap())
                    .await?;
            }
        } else {
            self.client
                .get_object(lpath.to_str().unwrap(), rpath.to_str().unwrap())
                .await?;
        }

        Ok(())
    }

    pub async fn rm(&mut self, path: &Path, recursive: bool) -> Result<(), StorageError> {
        if recursive {
            self.client.delete_objects(path.to_str().unwrap()).await?;
        } else {
            self.client.delete_object(path.to_str().unwrap()).await?;
        }

        Ok(())
    }

    pub async fn exists(&mut self, path: &Path) -> Result<bool, StorageError> {
        let objects = self.client.find(path.to_str().unwrap()).await?;

        Ok(!objects.is_empty())
    }

    pub async fn put(
        &mut self,
        lpath: &Path,
        rpath: &Path,
        recursive: bool,
    ) -> Result<(), StorageError> {
        if recursive {
            if !lpath.is_dir() {
                return Err(StorageError::Error(
                    "Local path must be a directory for recursive put".to_string(),
                ));
            }

            let files: Vec<PathBuf> = get_files(lpath)?;

            for file in files {
                let stripped_lpath_clone = lpath;
                let stripped_rpath_clone = rpath;
                let stripped_file_path = file.clone();

                let relative_path = file.relative_path(stripped_lpath_clone)?;
                let remote_path = stripped_rpath_clone.join(relative_path);

                let mut uploader = self
                    .client
                    .create_multipart_uploader(&remote_path, stripped_lpath_clone)
                    .await?;

                uploader.upload_file_in_chunks(&stripped_file_path).await?;
            }

            Ok(())
        } else {
            let mut uploader = self.client.create_multipart_uploader(rpath, lpath).await?;
            uploader.upload_file_in_chunks(lpath).await?;

            Ok(())
        }
    }

    pub async fn generate_presigned_url(&mut self, path: &Path) -> Result<String, StorageError> {
        self.client
            .generate_presigned_url(path.to_str().unwrap())
            .await
    }
}

#[pyclass]
pub struct PyHttpFSStorageClient {
    pub client: HttpStorageClient,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyHttpFSStorageClient {
    #[new]
    fn new(settings: &mut OpsmlStorageSettings) -> AnyhowResult<Self> {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let client = rt
            .block_on(async {
                let client = build_http_client(&settings.api_settings).context(
                    LogColors::green("Error occurred while building HTTP client"),
                )?;

                HttpStorageClient::new(settings, &client)
                    .await
                    .context(LogColors::green(
                        "Error occurred while creating HTTP storage client",
                    ))
            })
            .context(LogColors::green(
                "Error occurred while creating HTTP storage client",
            ))?;

        Ok(Self {
            client,
            runtime: rt,
        })
    }

    #[pyo3(signature = (path=PathBuf::new()))]
    fn find_info(&mut self, path: PathBuf) -> AnyhowResult<Vec<FileInfo>> {
        self.runtime.block_on(async {
            self.client
                .find_info(path.to_str().unwrap())
                .await
                .context(LogColors::green("Failed to list files"))
        })
    }

    #[pyo3(signature = (path=PathBuf::new()))]
    fn find(&mut self, path: PathBuf) -> Result<Vec<String>, StorageError> {
        self.runtime
            .block_on(async { self.client.find(path.to_str().unwrap()).await })
    }

    #[pyo3(signature = (lpath, rpath, recursive = false))]
    fn get(&mut self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError> {
        self.runtime.block_on(async {
            if recursive {
                let stripped_lpath_clone = lpath.clone();

                // list all objects in the path

                let objects = self.client.find(rpath.to_str().unwrap()).await?;

                // iterate over each object and get it
                for obj in objects {
                    let file_path = Path::new(obj.as_str());
                    let stripped_path = file_path;
                    let relative_path = file_path.relative_path(&rpath)?;
                    let local_path = stripped_lpath_clone.join(relative_path);

                    self.client
                        .get_object(
                            local_path.to_str().unwrap(),
                            stripped_path.to_str().unwrap(),
                        )
                        .await?;
                }
            } else {
                self.client
                    .get_object(lpath.to_str().unwrap(), rpath.to_str().unwrap())
                    .await?;
            }

            Ok(())
        })
    }

    #[pyo3(signature = (lpath, rpath, recursive = false))]
    fn put(&mut self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError> {
        self.runtime.block_on(async {
            if recursive {
                if !lpath.is_dir() {
                    return Err(StorageError::Error(
                        "Local path must be a directory for recursive put".to_string(),
                    ));
                }

                let files: Vec<PathBuf> = get_files(&lpath)?;

                for file in files {
                    let stripped_lpath_clone = lpath.clone();
                    let stripped_rpath_clone = rpath.clone();
                    let stripped_file_path = file.clone();

                    let relative_path = file.relative_path(&stripped_lpath_clone)?;
                    let remote_path = stripped_rpath_clone.join(relative_path);

                    let mut uploader = self
                        .client
                        .create_multipart_uploader(&remote_path, &lpath)
                        .await?;

                    uploader.upload_file_in_chunks(&stripped_file_path).await?;
                }

                Ok(())
            } else {
                let mut uploader = self
                    .client
                    .create_multipart_uploader(&rpath, &lpath)
                    .await?;

                uploader.upload_file_in_chunks(&lpath).await?;
                Ok(())
            }
        })
    }

    #[pyo3(signature = (path, recursive = false))]
    fn rm(&mut self, path: PathBuf, recursive: bool) -> Result<(), StorageError> {
        self.runtime.block_on(async {
            if recursive {
                self.client.delete_objects(path.to_str().unwrap()).await?;
            } else {
                self.client.delete_object(path.to_str().unwrap()).await?;
            }

            Ok(())
        })
    }

    fn exists(&mut self, path: PathBuf) -> Result<bool, StorageError> {
        let objects = self
            .runtime
            .block_on(async { self.client.find(path.to_str().unwrap()).await })?;

        Ok(!objects.is_empty())
    }

    fn generate_presigned_url(&mut self, path: PathBuf) -> Result<String, StorageError> {
        self.runtime.block_on(async {
            self.client
                .generate_presigned_url(path.to_str().unwrap())
                .await
        })
    }
}

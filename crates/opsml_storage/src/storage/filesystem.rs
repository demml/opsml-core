use crate::storage::enums::client::StorageClientEnum;
use crate::storage::http::client::HttpFSStorageClient;
use async_trait::async_trait;
use opsml_contracts::FileInfo;
use opsml_error::error::StorageError;
use opsml_settings::config::{OpsmlStorageSettings, StorageType};
use pyo3::prelude::*;
use std::path::Path;
use std::path::PathBuf;

#[async_trait]
pub trait FileSystem {
    fn name(&self) -> &str;
    fn storage_type(&self) -> StorageType;
    async fn new(settings: &OpsmlStorageSettings) -> Self;
    async fn find(&self, path: &Path) -> Result<Vec<String>, StorageError>;
    async fn find_info(&self, path: &Path) -> Result<Vec<FileInfo>, StorageError>;
    async fn get(&self, lpath: &Path, rpath: &Path, recursive: bool) -> Result<(), StorageError>;
    async fn put(&self, lpath: &Path, rpath: &Path, recursive: bool) -> Result<(), StorageError>;
    async fn copy(&self, src: &Path, dest: &Path, recursive: bool) -> Result<(), StorageError>;
    async fn rm(&self, path: &Path, recursive: bool) -> Result<(), StorageError>;
    async fn exists(&self, path: &Path) -> Result<bool, StorageError>;
    async fn generate_presigned_url(
        &self,
        path: &Path,
        expiration: u64,
    ) -> Result<String, StorageError>;
}

pub struct FileSystemStorage {
    fs: Option<StorageClientEnum>,
    http: Option<HttpFSStorageClient>,
    client_mode: bool,
}

impl FileSystemStorage {
    pub async fn new(settings: &mut OpsmlStorageSettings) -> Result<Self, StorageError> {
        if !settings.client_mode {
            Ok(FileSystemStorage {
                fs: Some(StorageClientEnum::new(settings).await?),
                http: None,
                client_mode: settings.client_mode,
            })
        } else {
            Ok(FileSystemStorage {
                fs: None,
                http: Some(HttpFSStorageClient::new(&mut *settings).await?),
                client_mode: settings.client_mode,
            })
        }
    }

    pub fn name(&self) -> &str {
        if self.client_mode {
            self.http.as_ref().unwrap().name()
        } else {
            self.fs.as_ref().unwrap().name()
        }
    }

    pub fn storage_type(&self) -> StorageType {
        if self.client_mode {
            self.http.as_ref().unwrap().storage_type()
        } else {
            self.fs.as_ref().unwrap().storage_type()
        }
    }

    pub async fn find(&mut self, path: &Path) -> Result<Vec<String>, StorageError> {
        if self.client_mode {
            self.http.as_mut().unwrap().find(path).await
        } else {
            self.fs.as_ref().unwrap().find(path).await
        }
    }

    pub async fn find_info(&mut self, path: &Path) -> Result<Vec<FileInfo>, StorageError> {
        if self.client_mode {
            self.http.as_mut().unwrap().find_info(path).await
        } else {
            self.fs.as_ref().unwrap().find_info(path).await
        }
    }

    pub async fn get(
        &mut self,
        lpath: &Path,
        rpath: &Path,
        recursive: bool,
    ) -> Result<(), StorageError> {
        if self.client_mode {
            self.http
                .as_mut()
                .unwrap()
                .get(lpath, rpath, recursive)
                .await
        } else {
            self.fs.as_ref().unwrap().get(lpath, rpath, recursive).await
        }
    }

    pub async fn put(
        &mut self,
        lpath: &Path,
        rpath: &Path,
        recursive: bool,
    ) -> Result<(), StorageError> {
        if self.client_mode {
            self.http
                .as_mut()
                .unwrap()
                .put(lpath, rpath, recursive)
                .await
        } else {
            self.fs.as_ref().unwrap().put(lpath, rpath, recursive).await
        }
    }

    pub async fn rm(&mut self, path: &Path, recursive: bool) -> Result<(), StorageError> {
        if self.client_mode {
            self.http.as_mut().unwrap().rm(path, recursive).await
        } else {
            self.fs.as_ref().unwrap().rm(path, recursive).await
        }
    }

    pub async fn exists(&mut self, path: &Path) -> Result<bool, StorageError> {
        if self.client_mode {
            self.http.as_mut().unwrap().exists(path).await
        } else {
            self.fs.as_ref().unwrap().exists(path).await
        }
    }

    pub async fn generate_presigned_url(
        &mut self,
        path: &Path,
        expiration: u64,
    ) -> Result<String, StorageError> {
        if self.client_mode {
            self.http
                .as_mut()
                .unwrap()
                .generate_presigned_url(path)
                .await
        } else {
            self.fs
                .as_ref()
                .unwrap()
                .generate_presigned_url(path, expiration)
                .await
        }
    }
}

#[pyclass]
pub struct PyFileSystemStorage {
    fs: Option<StorageClientEnum>,
    http: Option<HttpFSStorageClient>,
    client_mode: bool,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyFileSystemStorage {
    #[new]
    pub fn new(settings: &mut OpsmlStorageSettings) -> PyResult<Self> {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let (fs, http) = rt
            .block_on(async {
                let fs = if !settings.client_mode {
                    Some(StorageClientEnum::new(settings).await?)
                } else {
                    None
                };

                let http = if settings.client_mode {
                    Some(HttpFSStorageClient::new(&mut *settings).await?)
                } else {
                    None
                };

                Ok::<(Option<StorageClientEnum>, Option<HttpFSStorageClient>), StorageError>((
                    fs, http,
                ))
            })
            .unwrap();

        Ok(PyFileSystemStorage {
            fs,
            http,
            client_mode: settings.client_mode,
            runtime: rt,
        })
    }

    pub fn name(&self) -> &str {
        if self.client_mode {
            self.http.as_ref().unwrap().name()
        } else {
            self.fs.as_ref().unwrap().name()
        }
    }

    pub fn storage_type(&self) -> StorageType {
        if self.client_mode {
            self.http.as_ref().unwrap().storage_type()
        } else {
            self.fs.as_ref().unwrap().storage_type()
        }
    }

    #[pyo3(signature = (path=PathBuf::new()))]
    pub fn find(&mut self, path: PathBuf) -> PyResult<Vec<String>> {
        Ok(self
            .runtime
            .block_on(async {
                if self.client_mode {
                    self.http.as_mut().unwrap().find(&path).await
                } else {
                    self.fs.as_ref().unwrap().find(&path).await
                }
            })
            .unwrap())
    }

    #[pyo3(signature = (path=PathBuf::new()))]
    pub fn find_info(&mut self, path: PathBuf) -> PyResult<Vec<FileInfo>> {
        Ok(self
            .runtime
            .block_on(async {
                if self.client_mode {
                    self.http.as_mut().unwrap().find_info(&path).await
                } else {
                    self.fs.as_ref().unwrap().find_info(&path).await
                }
            })
            .unwrap())
    }

    #[pyo3(signature = (lpath, rpath, recursive = false))]
    pub fn get(&mut self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> PyResult<()> {
        self.runtime.block_on(async {
            if self.client_mode {
                self.http
                    .as_mut()
                    .unwrap()
                    .get(&lpath, &rpath, recursive)
                    .await
            } else {
                self.fs
                    .as_ref()
                    .unwrap()
                    .get(&lpath, &rpath, recursive)
                    .await
            }
        })?;
        Ok(())
    }

    #[pyo3(signature = (lpath, rpath, recursive = false))]
    pub fn put(&mut self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> PyResult<()> {
        self.runtime.block_on(async {
            if self.client_mode {
                self.http
                    .as_mut()
                    .unwrap()
                    .put(&lpath, &rpath, recursive)
                    .await
            } else {
                self.fs
                    .as_ref()
                    .unwrap()
                    .put(&lpath, &rpath, recursive)
                    .await
            }
        })?;
        Ok(())
    }

    #[pyo3(signature = (path, recursive = false))]
    pub fn rm(&mut self, path: PathBuf, recursive: bool) -> PyResult<()> {
        self.runtime.block_on(async {
            if self.client_mode {
                self.http.as_mut().unwrap().rm(&path, recursive).await
            } else {
                self.fs.as_ref().unwrap().rm(&path, recursive).await
            }
        })?;
        Ok(())
    }

    pub fn exists(&mut self, path: PathBuf) -> PyResult<bool> {
        Ok(self
            .runtime
            .block_on(async {
                if self.client_mode {
                    self.http.as_mut().unwrap().exists(&path).await
                } else {
                    self.fs.as_ref().unwrap().exists(&path).await
                }
            })
            .unwrap())
    }

    pub fn generate_presigned_url(&mut self, path: PathBuf, expiration: u64) -> PyResult<String> {
        Ok(self
            .runtime
            .block_on(async {
                if self.client_mode {
                    self.http
                        .as_mut()
                        .unwrap()
                        .generate_presigned_url(&path)
                        .await
                } else {
                    self.fs
                        .as_ref()
                        .unwrap()
                        .generate_presigned_url(&path, expiration)
                        .await
                }
            })
            .unwrap())
    }
}

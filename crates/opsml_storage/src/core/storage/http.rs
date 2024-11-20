use crate::core::http_client::client::HttpStorageClient;
use crate::core::storage::base::FileInfo;
use crate::core::storage::base::PathExt;
use crate::core::storage::base::StorageClient;
use crate::core::storage::base::{get_files, FileSystem, StorageSettings};
use crate::core::utils::error::StorageError;
use async_trait::async_trait;
use pyo3::prelude::*;
use std::path::{Path, PathBuf};

pub struct HttpFSStorageClient {
    client: HttpStorageClient,
}

#[async_trait]
impl FileSystem<HttpStorageClient> for HttpFSStorageClient {
    fn name(&self) -> &str {
        "HttpFSStorageClient"
    }

    fn client(&self) -> &HttpStorageClient {
        &self.client
    }

    async fn new(settings: StorageSettings) -> Self {
        HttpFSStorageClient {
            client: HttpStorageClient::new(settings).await.unwrap(),
        }
    }
}

impl HttpFSStorageClient {
    pub async fn put(
        &self,
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

            let files: Vec<PathBuf> = get_files(&lpath)?;

            for file in files {
                let stripped_lpath_clone = lpath;
                let stripped_rpath_clone = rpath;
                let stripped_file_path = file.clone();

                let relative_path = file.relative_path(&stripped_lpath_clone)?;
                let remote_path = stripped_rpath_clone.join(relative_path);

                let mut uploader = self.client.create_multipart_uploader(rpath).await?;

                self.client
                    .upload_file_in_chunks(&stripped_file_path, &remote_path, &mut uploader)
                    .await?;
            }

            Ok(())
        } else {
            let mut uploader = self.client.create_multipart_uploader(&rpath).await?;

            self.client
                .upload_file_in_chunks(&lpath, &rpath, &mut uploader)
                .await?;

            Ok(())
        }
    }
}

#[pyclass]
pub struct PyHttpFSStorageClient {
    client: HttpStorageClient,
    runtime: tokio::runtime::Runtime,
}

#[pymethods]
impl PyHttpFSStorageClient {
    #[new]
    fn new(settings: StorageSettings) -> Self {
        let rt = tokio::runtime::Runtime::new().unwrap();

        let client = rt.block_on(async { HttpStorageClient::new(settings).await.unwrap() });

        Self {
            client,
            runtime: rt,
        }
    }

    #[pyo3(signature = (path=PathBuf::new()))]
    fn find_info(&self, path: PathBuf) -> Result<Vec<FileInfo>, StorageError> {
        self.runtime
            .block_on(async { self.client.find_info(path.to_str().unwrap()).await })
    }

    #[pyo3(signature = (path=PathBuf::new()))]
    fn find(&self, path: PathBuf) -> Result<Vec<String>, StorageError> {
        self.runtime
            .block_on(async { self.client.find(path.to_str().unwrap()).await })
    }

    #[pyo3(signature = (lpath, rpath, recursive = false))]
    fn get(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError> {
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
    fn put(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError> {
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

                    let mut uploader = self.client.create_multipart_uploader(&remote_path).await?;

                    self.client
                        .upload_file_in_chunks(&stripped_file_path, &remote_path, &mut uploader)
                        .await?;
                }

                Ok(())
            } else {
                let mut uploader = self.client.create_multipart_uploader(&rpath).await?;

                self.client
                    .upload_file_in_chunks(&lpath, &rpath, &mut uploader)
                    .await?;
                Ok(())
            }
        })
    }

    #[pyo3(signature = (src, dest, recursive = false))]
    fn copy(&self, src: PathBuf, dest: PathBuf, recursive: bool) -> Result<(), StorageError> {
        self.runtime.block_on(async {
            if recursive {
                self.client
                    .copy_objects(src.to_str().unwrap(), dest.to_str().unwrap())
                    .await?;
            } else {
                self.client
                    .copy_object(src.to_str().unwrap(), dest.to_str().unwrap())
                    .await?;
            }

            Ok(())
        })
    }
    #[pyo3(signature = (path, recursive = false))]
    fn rm(&self, path: PathBuf, recursive: bool) -> Result<(), StorageError> {
        self.runtime.block_on(async {
            if recursive {
                self.client.delete_objects(path.to_str().unwrap()).await?;
            } else {
                self.client.delete_object(path.to_str().unwrap()).await?;
            }

            Ok(())
        })
    }

    fn exists(&self, path: PathBuf) -> Result<bool, StorageError> {
        let objects = self
            .runtime
            .block_on(async { self.client.find(path.to_str().unwrap()).await })?;

        Ok(!objects.is_empty())
    }

    #[pyo3(signature = (path, expiration = 600))]
    fn generate_presigned_url(
        &self,
        path: PathBuf,
        expiration: u64,
    ) -> Result<String, StorageError> {
        self.runtime.block_on(async {
            self.client
                .generate_presigned_url(path.to_str().unwrap(), expiration)
                .await
        })
    }
}

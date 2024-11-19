// create pyo3 async iterator
use crate::core::utils::error::StorageError;
use async_trait::async_trait;
use aws_smithy_types::byte_stream::ByteStream;
use futures::TryStream;
use pyo3::prelude::*;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
// take a stream of bytes

// create a method for Path that returns a relative path

#[derive(Debug, Clone)]
#[pyclass]
pub struct StorageSettings {
    #[pyo3(get)]
    pub storage_uri: String,

    #[pyo3(get)]
    pub using_client: bool,

    #[pyo3(get)]
    pub kwargs: HashMap<String, String>,
}

#[pymethods]
impl StorageSettings {
    #[new]
    pub fn new(storage_uri: String, using_client: bool, kwargs: HashMap<String, String>) -> Self {
        StorageSettings {
            storage_uri,
            using_client,
            kwargs,
        }
    }
}

impl Default for StorageSettings {
    fn default() -> Self {
        StorageSettings {
            storage_uri: "".to_string(),
            using_client: false,
            kwargs: HashMap::new(),
        }
    }
}

pub struct UploadPartArgs {
    pub chunk_size: u64,
    pub file_size: u64,
    pub path: PathBuf,
}

pub trait PathExt {
    fn relative_path(&self, base: &Path) -> Result<PathBuf, StorageError>;
    fn strip_path(&self, prefix: &str) -> PathBuf;
}

impl PathExt for Path {
    fn relative_path(&self, base: &Path) -> Result<PathBuf, StorageError> {
        let result = self
            .strip_prefix(base)
            .map_err(|e| StorageError::Error(format!("Failed to get relative path: {}", e)))
            .map(|p| p.to_path_buf());

        // if result is error, check if prefix occurs in the path (this happens with LocalStorageClient) and remove anything before the prefix and the prefix itself
        if result.is_err() {
            if let Some(pos) = self.iter().position(|part| part == base) {
                let relative_path: PathBuf = self.iter().skip(pos + 1).collect();
                return Ok(relative_path);
            } else {
                return Ok(PathBuf::from(self));
            }
        }

        result
    }

    fn strip_path(&self, prefix: &str) -> PathBuf {
        self.strip_prefix(prefix).unwrap_or(self).to_path_buf()
    }
}

/// Get all files in a directory (including subdirectories)
pub fn get_files(path: &Path) -> Result<Vec<PathBuf>, StorageError> {
    let files: Vec<_> = walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    Ok(files)
}

// Define the StorageClient trait with common methods
#[async_trait]
pub trait StorageClient: Sized {
    async fn bucket(&self) -> &str;
    async fn new(settings: StorageSettings) -> Result<Self, StorageError>;
    async fn find(&self, path: &str) -> Result<Vec<String>, StorageError>;
    async fn find_info(&self, path: &str) -> Result<Vec<FileInfo>, StorageError>;
    async fn get_object(&self, local_path: &str, remote_path: &str) -> Result<(), StorageError>;
    async fn upload_file_in_chunks(
        &self,
        local_path: &Path,
        remote_path: &Path,
        chunk_size: Option<u64>,
    ) -> Result<(), StorageError>;
    async fn copy_objects(&self, src: &str, dest: &str) -> Result<bool, StorageError>;
    async fn copy_object(&self, src: &str, dest: &str) -> Result<bool, StorageError>;
    async fn delete_objects(&self, path: &str) -> Result<bool, StorageError>;
    async fn delete_object(&self, path: &str) -> Result<bool, StorageError>;
    async fn generate_presigned_url(
        &self,
        path: &str,
        expiration: u64,
    ) -> Result<String, StorageError>;

    async fn put_stream_to_object<S>(&self, path: &str, stream: S) -> Result<(), StorageError>
    where
        S: TryStream + Send + Sync + Unpin + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
        ByteStream: From<S>;
}

#[async_trait]
pub trait FileSystem<T: StorageClient> {
    fn name(&self) -> &str;
    fn client(&self) -> &T;
    async fn new(settings: StorageSettings) -> Self;

    async fn find(&self, path: &Path) -> Result<Vec<String>, StorageError> {
        let stripped_path = path.strip_path(self.client().bucket().await);
        self.client().find(stripped_path.to_str().unwrap()).await
    }

    async fn find_info(&self, path: &Path) -> Result<Vec<FileInfo>, StorageError> {
        let stripped_path = path.strip_path(self.client().bucket().await);
        self.client()
            .find_info(stripped_path.to_str().unwrap())
            .await
    }

    async fn get(&self, lpath: &Path, rpath: &Path, recursive: bool) -> Result<(), StorageError> {
        // strip the paths
        let stripped_rpath = rpath.strip_path(self.client().bucket().await);
        let stripped_lpath = lpath.strip_path(self.client().bucket().await);

        if recursive {
            let stripped_lpath_clone = stripped_lpath.clone();

            // list all objects in the path
            let objects = self.client().find(stripped_rpath.to_str().unwrap()).await?;

            // iterate over each object and get it
            for obj in objects {
                let file_path = Path::new(obj.as_str());
                let stripped_path = file_path.strip_path(self.client().bucket().await);
                let relative_path = file_path.relative_path(&stripped_rpath)?;
                let local_path = stripped_lpath_clone.join(relative_path);

                self.client()
                    .get_object(
                        local_path.to_str().unwrap(),
                        stripped_path.to_str().unwrap(),
                    )
                    .await?;
            }
        } else {
            self.client()
                .get_object(
                    stripped_lpath.to_str().unwrap(),
                    stripped_rpath.to_str().unwrap(),
                )
                .await?;
        }

        Ok(())
    }
    async fn put(&self, lpath: &Path, rpath: &Path, recursive: bool) -> Result<(), StorageError> {
        let stripped_lpath = lpath.strip_path(self.client().bucket().await);
        let stripped_rpath = rpath.strip_path(self.client().bucket().await);

        if recursive {
            if !stripped_lpath.is_dir() {
                return Err(StorageError::Error(
                    "Local path must be a directory for recursive put".to_string(),
                ));
            }

            let files: Vec<PathBuf> = get_files(&stripped_lpath)?;

            for file in files {
                let stripped_lpath_clone = stripped_lpath.clone();
                let stripped_rpath_clone = stripped_rpath.clone();
                let stripped_file_path = file.strip_path(self.client().bucket().await);

                let relative_path = file.relative_path(&stripped_lpath_clone)?;
                let remote_path = stripped_rpath_clone.join(relative_path);

                self.client()
                    .upload_file_in_chunks(&stripped_file_path, &remote_path, None)
                    .await?;
            }

            Ok(())
        } else {
            self.client()
                .upload_file_in_chunks(&stripped_lpath, &stripped_rpath, None)
                .await?;
            Ok(())
        }
    }
    async fn copy(&self, src: &Path, dest: &Path, recursive: bool) -> Result<(), StorageError> {
        let stripped_src = src.strip_path(self.client().bucket().await);
        let stripped_dest = dest.strip_path(self.client().bucket().await);

        if recursive {
            self.client()
                .copy_objects(
                    stripped_src.to_str().unwrap(),
                    stripped_dest.to_str().unwrap(),
                )
                .await?;
        } else {
            self.client()
                .copy_object(
                    stripped_src.to_str().unwrap(),
                    stripped_dest.to_str().unwrap(),
                )
                .await?;
        }

        Ok(())
    }
    async fn rm(&self, path: &Path, recursive: bool) -> Result<(), StorageError> {
        let stripped_path = path.strip_path(self.client().bucket().await);

        if recursive {
            self.client()
                .delete_objects(stripped_path.to_str().unwrap())
                .await?;
        } else {
            self.client()
                .delete_object(stripped_path.to_str().unwrap())
                .await?;
        }

        Ok(())
    }
    async fn exists(&self, path: &Path) -> Result<bool, StorageError> {
        let stripped_path = path.strip_path(self.client().bucket().await);
        let objects = self.client().find(stripped_path.to_str().unwrap()).await?;

        Ok(!objects.is_empty())
    }

    async fn generate_presigned_url(
        &self,
        path: &Path,
        expiration: u64,
    ) -> Result<String, StorageError> {
        let stripped_path = path.strip_path(self.client().bucket().await);
        self.client()
            .generate_presigned_url(stripped_path.to_str().unwrap(), expiration)
            .await
    }
    async fn put_stream<S>(&self, path: &Path, stream: S) -> Result<(), StorageError>
    where
        S: TryStream + Send + Sync + Unpin + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
        ByteStream: From<S>,
    {
        let stripped_path = path.strip_path(self.client().bucket().await);
        self.client()
            .put_stream_to_object(stripped_path.to_str().unwrap(), stream)
            .await
    }
}

#[derive(Debug)]
#[pyclass]
pub struct FileInfo {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub size: i64,
    #[pyo3(get)]
    pub object_type: String,
    #[pyo3(get)]
    pub created: String,
    #[pyo3(get)]
    pub suffix: String,
}

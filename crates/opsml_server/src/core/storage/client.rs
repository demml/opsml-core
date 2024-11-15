use opsml_storage::core::storage::base::FileInfo;
use opsml_storage::core::storage::base::FileSystem;
use opsml_storage::core::storage::local::LocalFSStorageClient;
use opsml_storage::core::utils::error::StorageError;
use std::path::Path;

#[cfg(feature = "google_storage")]
use opsml_storage::core::storage::google::google_storage::GCSFSStorageClient;

#[cfg(feature = "aws_storage")]
use opsml_storage::core::storage::aws::aws_storage::S3FStorageClient;

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
    pub async fn new(storage_uri: String) -> Result<Self, StorageError> {
        // match start of storage uri with starts_with("gs://") or starts_with("s3://")
        // to determine the storage type

        match storage_uri {
            #[cfg(feature = "google_storage")]
            _ if storage_uri.starts_with("gs://") => {
                // strip the gs:// prefix
                let bucket = storage_uri.strip_prefix("gs://").unwrap().to_string();
                let client = GCSFSStorageClient::new(bucket).await;
                Ok(StorageClientEnum::Google(client))
            }
            #[cfg(feature = "aws_storage")]
            _ if storage_uri.starts_with("s3://") => {
                // strip the s3:// prefix
                let bucket = storage_uri.strip_prefix("s3://").unwrap().to_string();
                let client = S3FStorageClient::new(bucket).await;
                Ok(StorageClientEnum::AWS(client))
            }
            _ => {
                let client = LocalFSStorageClient::new(storage_uri).await;
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
}

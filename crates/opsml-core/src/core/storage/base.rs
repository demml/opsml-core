use crate::core::utils::error::StorageError;
use bytes::Bytes;
use futures::{Stream, TryStream};

#[cfg(feature = "google_storage")]
use super::google::google_storage;
/// Generic interface for storage backends.

#[derive(Clone)]
pub enum Client {
    #[cfg(feature = "google_storage")]
    GCS(google_storage::GoogleStorageClient),
}

impl Client {
    pub async fn new(client_type: ClientType, bucket: String) -> Result<Self, StorageError> {
        match client_type {
            #[cfg(feature = "google_storage")]
            ClientType::GCS => Ok(Client::GCS(
                google_storage::GoogleStorageClient::new(bucket).await?,
            )),
            #[allow(unreachable_patterns)]
            _ => Err(StorageError::UnsupportedClient),
        }
    }

    pub async fn get_object_stream(
        &self,
        rpath: &str,
    ) -> Result<impl Stream<Item = Result<Bytes, StorageError>>, StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            Client::GCS(client) => client.get_object_stream(rpath).await,
            #[allow(unreachable_patterns)]
            _ => Err(StorageError::UnsupportedClient),
        }
    }

    pub async fn find(&self, path: &str) -> Result<Vec<String>, StorageError> {
        match self {
            #[cfg(feature = "google_storage")]
            Client::GCS(client) => client.find(path).await,
            #[allow(unreachable_patterns)]
            _ => Err(StorageError::UnsupportedClient),
        }
    }

    pub async fn upload_stream_to_object<S>(
        &self,
        path: &str,
        stream: S,
    ) -> Result<String, StorageError>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        match self {
            #[cfg(feature = "google_storage")]
            Client::GCS(client) => client.upload_stream_to_object(path, stream).await,
            #[allow(unreachable_patterns)]
            _ => Err(StorageError::UnsupportedClient),
        }
    }
}

#[derive(Clone)]
pub enum ClientType {
    #[cfg(feature = "google_storage")]
    GCS,
}
// create a trait that will return the correct storage client

#[async_trait::async_trait]
pub trait StorageClientTrait {
    async fn get_object_stream(
        &self,
        rpath: &str,
    ) -> Result<impl Stream<Item = Result<Bytes, StorageError>>, StorageError>;
}

pub struct StorageClient {
    pub client: Client,
    pub client_type: ClientType,
}

impl StorageClient {
    /// Instantiate a new `StorageClient` with the specified `ClientType` and bucket.
    ///
    /// # Arguments
    ///
    /// * `client_type` - The type of client to use.
    /// * `bucket` - The bucket to use.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `StorageClient` if successful, or a `StorageError` if not.
    pub async fn new(client_type: ClientType, bucket: String) -> Result<Self, StorageError> {
        Ok(StorageClient {
            client: Client::new(client_type.clone(), bucket).await?,
            client_type: client_type,
        })
    }

    /// Generic method to get a stream of bytes from a remote object.
    /// The stream will return `Bytes` chunks.
    ///
    /// # Arguments
    ///
    /// * `rpath` - The remote path to the object.
    ///
    /// # Returns
    ///
    /// A stream of `Result<Bytes, StorageError>`.
    pub async fn get_object_stream(
        &self,
        rpath: &str,
    ) -> Result<impl Stream<Item = Result<Bytes, StorageError>>, StorageError> {
        self.client.get_object_stream(rpath).await
    }

    /// List all objects in a path
    ///
    /// # Arguments
    ///
    /// * `path` - The path to list objects from
    ///
    /// # Returns
    ///
    /// A list of objects in the path
    pub async fn find(&self, path: &str) -> Result<Vec<String>, StorageError> {
        self.client.find(path).await
    }
}

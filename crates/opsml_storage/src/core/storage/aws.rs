use crate::core::utils::error::StorageError;
use aws_config::BehaviorVersion;
use aws_config::SdkConfig;
use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_sdk_s3::Client;

#[derive(Clone)]
pub struct AWSCreds {
    pub config: SdkConfig,
}

impl AWSCreds {
    pub async fn new() -> Result<Self, StorageError> {
        let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

        Ok(Self { config })
    }
}

pub struct AWSStorageClient {
    pub client: Client,
    pub bucket: String,
}

impl AWSStorageClient {
    pub async fn new(bucket: String) -> Result<Self, StorageError> {
        let creds = AWSCreds::new().await?;
        let client = Client::new(&creds.config);
        Ok(Self { client, bucket })
    }

    pub async fn get_object_stream(&self, rpath: &str) -> Result<GetObjectOutput, StorageError> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(rpath)
            .send()
            .await
            .map_err(|e| StorageError::Error(format!("Failed to get object stream: {}", e)))?;
        Ok(response)
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
        let response = self
            .client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(path)
            .send()
            .await
            .map_err(|e| StorageError::Error(format!("Failed to list objects: {}", e)))?;

        Ok(response
            .contents
            .unwrap_or_else(|| Vec::new())
            .iter()
            .filter_map(|o| o.key.clone())
            .collect())
    }

    /// copy object from one bucket to another without deleting the source object
    ///
    /// # Arguments
    ///
    /// * `src` - The path to the source object
    /// * `dest` - The path to the destination object
    ///
    /// # Returns
    ///
    /// A Result with the object name if successful
    pub async fn copy_object(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
        self.client
            .copy_object()
            .copy_source(format!("{}/{}", self.bucket, src))
            .bucket(&self.bucket)
            .key(dest)
            .send()
            .await
            .map_err(|e| StorageError::Error(format!("Failed to copy object: {}", e)))?;
        Ok(true)
    }
}

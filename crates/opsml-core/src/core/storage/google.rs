#[cfg(feature = "google_storage")]
pub mod google_storage {
    use crate::core::utils::error::StorageError;
    use base64::prelude::*;
    use bytes::Bytes;
    use futures::stream::Stream;
    use futures::TryStream;
    use futures::TryStreamExt;
    use google_cloud_auth::credentials::CredentialsFile;
    use google_cloud_storage::client::{Client, ClientConfig};
    use google_cloud_storage::http::objects::download::Range;
    use google_cloud_storage::http::objects::get::GetObjectRequest;
    use google_cloud_storage::http::objects::list::ListObjectsRequest;
    use google_cloud_storage::http::objects::upload::UploadType;
    use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest};
    use google_cloud_storage::http::resumable_upload_client::ChunkSize;
    use google_cloud_storage::http::resumable_upload_client::ResumableUploadClient;
    use google_cloud_storage::http::resumable_upload_client::UploadStatus;
    use pyo3::prelude::*;
    use serde_json::Value;
    use std::env;
    use std::path::PathBuf;
    use tokio::runtime::Runtime;

    #[derive(Clone)]
    pub struct GcpCreds {
        pub creds: Option<CredentialsFile>,
        pub project: Option<String>,
        pub default_creds: bool,
        pub service_account: Option<Value>,
    }

    impl GcpCreds {
        pub async fn new() -> Result<Self, StorageError> {
            let mut creds = GcpCreds {
                creds: None,
                project: None,
                default_creds: false,
                service_account: None,
            };
            creds
                .check_model()
                .await
                .map_err(|e| StorageError::Error(format!("Unable to check model: {}", e)))?;
            Ok(creds)
        }

        async fn check_model(&mut self) -> Result<(), StorageError> {
            if let Ok(base64_creds) = env::var("GOOGLE_ACCOUNT_JSON_BASE64") {
                let cred_string = self
                    .decode_base64(&base64_creds)
                    .map_err(|e| StorageError::Error(format!("Unable to decode base64: {}", e)))?;

                self.creds = Some(CredentialsFile::new_from_str(&cred_string).await.map_err(
                    |e| {
                        StorageError::Error(format!(
                            "Unable to create credentials file from string: {}",
                            e
                        ))
                    },
                )?);
            }

            if let Ok(_service_account_file) = env::var("GOOGLE_APPLICATION_CREDENTIALS_JSON") {
                self.creds = Some(CredentialsFile::new().await.map_err(|e| {
                    StorageError::Error(format!(
                        "Unable to create credentials file from file: {}",
                        e
                    ))
                })?);
            }

            Ok(())
        }

        fn decode_base64(&mut self, service_base64_creds: &str) -> Result<String, StorageError> {
            let decoded = BASE64_STANDARD
                .decode(service_base64_creds)
                .map_err(|e| StorageError::Error(format!("Unable to decode base64: {}", e)))?;
            let decoded_str = String::from_utf8(decoded)
                .map_err(|e| StorageError::Error(format!("Unable to convert to string: {}", e)))?;
            Ok(decoded_str)
        }
    }

    #[derive(Clone)]
    pub struct GoogleResumableUploadClient {
        pub client: ResumableUploadClient,
        first_byte: u64,
        last_byte: u64,
        total_size: u64,
        chunk_size: u64,
    }

    impl GoogleResumableUploadClient {
        pub async fn upload_multiple_chunks(
            &mut self,
            chunk: bytes::Bytes,
        ) -> Result<Option<String>, StorageError> {
            let size = ChunkSize::new(self.first_byte, self.last_byte, Some(self.total_size));
            let result = self
                .client
                .upload_multiple_chunk(chunk, &size)
                .await
                .map_err(|e| {
                    StorageError::Error(format!(
                        "Unable to upload multiple chunks to resumable upload: {}",
                        e
                    ))
                })?;

            // extract the range from the result and update the first_byte and last_byte
            // check if enum is Ok
            match result {
                UploadStatus::Ok(object) => return Ok(Some(object.name)),
                UploadStatus::ResumeIncomplete(range) => {
                    self.first_byte = range.last_byte + 1;

                    // if the last_byte + chunk_size is greater than the total size, set the last_byte to the total size
                    if range.last_byte + self.chunk_size > self.total_size {
                        self.last_byte = self.total_size;
                    } else {
                        self.last_byte = range.last_byte + self.chunk_size;
                    }
                }
                UploadStatus::NotStarted => {
                    self.first_byte = 0;
                    self.last_byte = self.chunk_size;
                }
            }

            Ok(None)
        }
    }

    #[derive(Clone)]
    pub struct GoogleStorageClient {
        pub client: Client,
        pub bucket: String,
    }

    impl GoogleStorageClient {
        pub async fn new(bucket: String) -> Result<Self, StorageError> {
            let client = GoogleStorageClient::build_client().await?;

            Ok(GoogleStorageClient { client, bucket })
        }

        pub async fn build_client() -> Result<Client, StorageError> {
            let creds = GcpCreds::new().await?;
            // If no credentials, attempt to create a default client pulling from the environment
            if creds.creds.is_none() {
                let config = ClientConfig::default().with_auth().await;

                // if error, use ClientConfig::default().anonymous();
                let config = match config {
                    Ok(config) => config,
                    Err(_) => ClientConfig::default().anonymous(),
                };

                return Ok(Client::new(config));

            // if creds are set (base64 for JSON file)
            } else {
                let config = ClientConfig::default()
                    .with_credentials(creds.creds.unwrap())
                    .await
                    .map_err(|e| {
                        StorageError::Error(format!(
                            "Unable to create client with credentials: {}",
                            e
                        ))
                    })?;

                let client = Client::new(config);
                return Ok(client);
            }
        }

        /// Get an object from the storage bucket and return a stream of bytes to pass to
        /// an async iterator
        ///
        /// # Arguments
        ///
        /// * `rpath` - The path to the object in the bucket
        ///
        /// # Returns
        ///
        /// A stream of bytes
        pub async fn get_object_stream(
            &self,
            rpath: &str,
        ) -> Result<impl Stream<Item = Result<Bytes, StorageError>>, StorageError> {
            // open a bucket and blob and return the stream
            let result = self
                .client
                .download_streamed_object(
                    &GetObjectRequest {
                        bucket: self.bucket.clone(),
                        object: rpath.to_string(),
                        ..Default::default()
                    },
                    &Range::default(),
                )
                .await
                .map_err(|e| StorageError::Error(format!("Unable to download object: {}", e)))?;

            Ok(result
                .map_err(|e| StorageError::Error(format!("Error while streaming object: {}", e))))
        }

        /// upload stream to google cloud storage object. This method will take an async iterator and upload it in chunks to the object
        ///
        /// # Arguments
        ///
        /// * `path` - The path to the object in the bucket
        /// * `stream` - The stream of bytes to upload
        ///
        /// # Returns
        ///
        /// A Result with the object name if successful

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
            let filename = path.to_string();
            let object_file = Media::new(filename);
            let upload_type = UploadType::Simple(object_file);

            let result = self
                .client
                .upload_streamed_object(
                    &UploadObjectRequest {
                        bucket: self.bucket.clone(),

                        ..Default::default()
                    },
                    stream,
                    &upload_type,
                )
                .await
                .map_err(|e| StorageError::Error(format!("Unable to upload object: {}", e)))?;

            Ok(result.name)
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
            let result = self
                .client
                .list_objects(&ListObjectsRequest {
                    bucket: self.bucket.clone(),
                    prefix: Some(path.to_string()),
                    ..Default::default()
                })
                .await
                .map_err(|e| StorageError::Error(format!("Unable to list objects: {}", e)))?;

            // return a list of object names if results.items is not None, Else return empty list
            Ok(result
                .items
                .unwrap_or_else(|| vec![])
                .iter()
                .map(|o| o.name.clone())
                .collect())
        }

        /// Create a resumable upload session
        ///
        /// # Arguments
        ///
        /// * `path` - The path to the object in the bucket
        ///
        ///
        pub async fn create_resumable_upload_session(
            &self,
            path: &str,
            chunk_size: u64,
            total_size: u64,
        ) -> Result<GoogleResumableUploadClient, StorageError> {
            let filename = path.to_string();
            let object_file = Media::new(filename);
            let upload_type = UploadType::Simple(object_file);

            let result = self
                .client
                .prepare_resumable_upload(
                    &UploadObjectRequest {
                        bucket: self.bucket.clone(),

                        ..Default::default()
                    },
                    &upload_type,
                )
                .await
                .map_err(|e| StorageError::Error(format!("Unable to upload object: {}", e)))?;

            Ok(GoogleResumableUploadClient {
                client: result,
                first_byte: 0,
                last_byte: chunk_size,
                total_size,
                chunk_size: chunk_size,
            })
        }
    }

    #[pyclass]
    pub struct GCSFSStorageClient {
        client: GoogleStorageClient,
    }

    #[pymethods]
    impl GCSFSStorageClient {
        #[new]
        pub fn new(bucket: String) -> Self {
            let client = Runtime::new()
                .unwrap()
                .block_on(GoogleStorageClient::new(bucket))
                .unwrap();

            GCSFSStorageClient { client }
        }

        pub async fn find(&self, path: PathBuf) -> Vec<String> {
            // remove bucket from path
            let stripped_path = path
                .strip_prefix(&self.client.bucket)
                .unwrap()
                .to_str()
                .unwrap();

            self.client.find(&stripped_path).await.unwrap()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::io::Write;
        use tempfile::NamedTempFile;
        use tokio;

        fn setup() {
            unsafe {
                env::remove_var("GOOGLE_ACCOUNT_JSON_BASE64");
                env::remove_var("GOOGLE_APPLICATION_CREDENTIALS_JSON");
            }
        }

        #[tokio::test]
        async fn test_create_client_with_anonymous() {
            setup();
            let client = GoogleStorageClient::new("bucket".to_string()).await;

            assert!(client.is_ok());
        }

        #[tokio::test]
        async fn test_create_client_with_google_application_credentials_json() {
            setup();

            // Create a temporary file with fake credentials
            let mut file = NamedTempFile::new().expect("Unable to create temp file");
            let fake_credentials = r#"
        {
            "type": "service_account",
            "project_id": "fake-project-id",
            "private_key_id": "fake-private-key-id",
            "private_key": "-----BEGIN PRIVATE KEY-----\nFAKEPRIVATEKEY\n-----END PRIVATE KEY-----\n",
            "client_email": "fake-email@fake-project-id.iam.gserviceaccount.com",
            "client_id": "fake-client-id",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token",
            "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
            "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/fake-email%40fake-project-id.iam.gserviceaccount.com"
        }
        "#;
            file.write_all(fake_credentials.as_bytes())
                .expect("Unable to write to temp file");

            unsafe {
                // Set the environment variable to the path of the temp file
                env::set_var("GOOGLE_APPLICATION_CREDENTIALS_JSON", file.path());
            }

            let client = GoogleStorageClient::new("bucket".to_string()).await;

            // this will fail because the credentials are fake. We are just testing the client creation
            assert!(client.is_err());
        }

        #[tokio::test]
        async fn test_create_client_with_google_account_json_base64() {
            setup();

            // Create a base64 encoded string of fake credentials
            let fake_credentials = r#"
        {
            "type": "service_account",
            "project_id": "fake-project-id",
            "private_key_id": "fake-private-key-id",
            "private_key": "-----BEGIN PRIVATE KEY-----\nFAKEPRIVATEKEY\n-----END PRIVATE KEY-----\n",
            "client_email": "fake-email@fake-project-id.iam.gserviceaccount.com",
            "client_id": "fake-client-id",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token",
            "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
            "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/fake-email%40fake-project-id.iam.gserviceaccount.com"
        }
        "#;
            let base64_credentials = BASE64_STANDARD.encode(fake_credentials);

            unsafe {
                // Set the environment variable to the base64 encoded string
                env::set_var("GOOGLE_ACCOUNT_JSON_BASE64", base64_credentials);
            }

            let client = GoogleStorageClient::new("bucket".to_string()).await;

            // this will fail because the credentials are fake. We are just testing the client creation
            assert!(client.is_err());
        }
    }
}

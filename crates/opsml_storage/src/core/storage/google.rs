#[cfg(feature = "google_storage")]
pub mod google_storage {

    use crate::core::storage::base::FileInfo;
    use crate::core::storage::base::FileSystem;
    use crate::core::storage::base::PathExt;
    use crate::core::storage::base::StorageClient;
    use crate::core::utils::error::StorageError;
    use aws_smithy_types::byte_stream::ByteStream;
    use aws_smithy_types::byte_stream::Length;
    use base64::prelude::*;
    use futures::stream::Stream;
    use futures::StreamExt;
    use futures::TryStream;
    use google_cloud_auth::credentials::CredentialsFile;
    use google_cloud_storage::client::{Client, ClientConfig};
    use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
    use google_cloud_storage::http::objects::download::Range;
    use google_cloud_storage::http::objects::get::GetObjectRequest;
    use google_cloud_storage::http::objects::list::ListObjectsRequest;
    use google_cloud_storage::http::objects::upload::UploadType;
    use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest};
    use google_cloud_storage::http::objects::Object;
    use google_cloud_storage::http::resumable_upload_client::ChunkSize;
    use google_cloud_storage::http::resumable_upload_client::ResumableUploadClient;
    use google_cloud_storage::http::resumable_upload_client::UploadStatus;
    use google_cloud_storage::sign::SignedURLMethod;
    use google_cloud_storage::sign::SignedURLOptions;
    use pyo3::prelude::*;
    use serde_json::Value;
    use std::env;
    use std::fs::File;
    use std::io::BufRead;
    use std::io::BufReader;
    use std::io::Read;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;
    use tokio::runtime::Handle;
    use tokio::runtime::Runtime;
    use walkdir;

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

            if let Ok(_service_account_file) = env::var("GOOGLE_APPLICATION_CREDENTIALS_JSON")
                .or_else(|_| env::var("GOOGLE_APPLICATION_CREDENTIALS"))
            {
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

    pub struct GoogleMultipartUploadClient {
        pub client: ResumableUploadClient,
        file_completed: bool,
        handle: tokio::runtime::Handle,
    }

    impl GoogleMultipartUploadClient {
        pub fn get_next_chunk(
            &self,
            path: &Path,
            chunk_size: u64,
            chunk_index: u64,
            this_chunk_size: u64,
        ) -> Result<ByteStream, StorageError> {
            let handle = self.handle.clone();
            handle.block_on(async {
                let stream = ByteStream::read_from()
                    .path(path)
                    .offset(chunk_index * chunk_size)
                    .length(Length::Exact(this_chunk_size))
                    .build()
                    .await
                    .map_err(|e| StorageError::Error(format!("Failed to get next chunk: {}", e)))?;

                Ok(stream)
            })
        }
        pub fn upload_part(
            &mut self,
            chunk: ByteStream,
            first_byte: u64,
            last_byte: u64,
            total_size: u64,
        ) -> Result<UploadStatus, StorageError> {
            let handle = self.handle.clone();
            let size = ChunkSize::new(first_byte, last_byte, Some(total_size));

            handle.block_on(async {
                let data = chunk
                    .collect()
                    .await
                    .map_err(|e| {
                        StorageError::Error(format!("Unable to collect chunk data: {}", e))
                    })?
                    .into_bytes();

                println!("Uploading chunk: {:?}", size);
                println!("Data size: {:?}", data.len());

                let result = self
                    .client
                    .upload_multiple_chunk(data, &size)
                    .await
                    .map_err(|e| {
                        StorageError::Error(format!(
                            "Unable to upload multiple chunks to resumable upload: {}",
                            e
                        ))
                    })?;

                Ok(result)
            })
        }
    }

    pub struct GoogleStorageClient {
        pub client: Client,
        pub bucket: String,
        runtime: tokio::runtime::Runtime,
    }

    impl StorageClient for GoogleStorageClient {
        fn bucket(&self) -> &str {
            &self.bucket
        }
        fn new(bucket: String) -> Self {
            let rt = Runtime::new().unwrap();

            let client: Result<Client, StorageError> = rt.block_on(async {
                let creds = GcpCreds::new().await?;
                // If no credentials, attempt to create a default client pulling from the environment
                if creds.creds.is_none() {
                    let config = ClientConfig::default().with_auth().await;

                    // if error, use ClientConfig::default().anonymous();
                    let config = match config {
                        Ok(config) => config,
                        Err(_) => ClientConfig::default().anonymous(),
                    };

                    Ok(Client::new(config))

                // if creds are set (base64 for JSON file)
                } else {
                    // try with credentials
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
                    Ok(client)
                }
            });

            match client {
                Ok(client) => GoogleStorageClient {
                    client,
                    bucket,
                    runtime: rt,
                },
                Err(e) => panic!("Unable to create GoogleStorageClient: {}", e),
            }
        }

        /// Download a remote object as a stream to a local file
        ///
        /// # Arguments
        ///
        /// * `lpath` - The path to the local file
        /// * `rpath` - The path to the remote file
        ///
        fn get_object(&self, lpath: &str, rpath: &str) -> Result<(), StorageError> {
            let mut stream = self.get_object_stream(rpath)?;

            // create and open lpath file
            let prefix = Path::new(lpath).parent().unwrap();

            if !prefix.exists() {
                // create the directory if it does not exist and skip errors
                std::fs::create_dir_all(prefix).map_err(|e| {
                    StorageError::Error(format!("Unable to create directory: {}", e))
                })?;
            }

            // create and open lpath file
            let mut file = File::create(lpath)
                .map_err(|e| StorageError::Error(format!("Unable to create file: {}", e)))?;

            self.runtime.block_on(async {
                // iterate over the stream and write to the file
                while let Some(v) = stream.next().await {
                    let chunk =
                        v.map_err(|e| StorageError::Error(format!("Stream error: {}", e)))?;
                    file.write_all(&chunk).map_err(|e| {
                        StorageError::Error(format!("Unable to write to file: {}", e))
                    })?;
                }

                Ok(())
            })
        }

        /// Generate a presigned url for an object in the storage bucket
        ///
        /// # Arguments
        ///
        /// * `path` - The path to the object in the bucket
        /// * `expiration` - The time in seconds for the presigned url to expire
        ///
        /// # Returns
        ///
        /// A Result with the presigned url if successful
        ///
        fn generate_presigned_url(
            &self,
            path: &str,
            expiration: u64,
        ) -> Result<String, StorageError> {
            self.runtime.block_on(async {
                let presigned_url = self
                    .client
                    .signed_url(
                        &self.bucket.clone(),
                        path,
                        None,
                        None,
                        SignedURLOptions {
                            method: SignedURLMethod::GET,
                            start_time: None,
                            expires: std::time::Duration::from_secs(expiration),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| {
                        StorageError::Error(format!("Unable to generate presigned url: {}", e))
                    })?;

                Ok(presigned_url)
            })
        }

        /// Upload file in chunks. This method will take a file path, open the file, read it in chunks and upload each chunk to the object
        ///
        /// # Arguments
        ///
        /// * `lpath` - The path to the file to upload
        /// * `rpath` - The path to the remote file
        /// * `chunk_size` - The size of each chunk
        ///
        /// # Returns
        ///
        /// A Result with the object name if successful
        ///
        fn upload_file_in_chunks(
            &self,
            lpath: &Path,
            rpath: &Path,
            chunk_size: Option<u64>,
        ) -> Result<(), StorageError> {
            let file = File::open(lpath)
                .map_err(|e| StorageError::Error(format!("Failed to open file: {}", e)))?;

            let metadata = file
                .metadata()
                .map_err(|e| StorageError::Error(format!("Failed to get file metadata: {}", e)))?;

            let file_size = metadata.len();

            // check chunk size. IF chunk size is None, set chunk size to 5MB. If chunk size is greater than file size, set chunk size to file size
            let mut chunk_size = chunk_size.unwrap_or(1024 * 1024 * 5);
            if chunk_size > file_size {
                chunk_size = file_size;
            }

            // calculate the number of parts
            let mut chunk_count = (file_size / chunk_size) + 1;
            let mut size_of_last_chunk = file_size % chunk_size;

            // if the last chunk is empty, reduce the number of parts
            if size_of_last_chunk == 0 {
                size_of_last_chunk = chunk_size;
                chunk_count -= 1;
            }

            let mut uploader = self.create_multipart_upload(rpath.to_str().unwrap())?;
            let mut status = UploadStatus::NotStarted;

            for chunk_index in 0..chunk_count {
                let this_chunk = if chunk_count - 1 == chunk_index {
                    size_of_last_chunk
                } else {
                    chunk_size
                };

                let first_byte = (chunk_index as u64) * this_chunk;
                let last_byte = first_byte + this_chunk - 1;

                let stream = uploader.get_next_chunk(lpath, chunk_size, chunk_index, this_chunk)?;
                status = uploader.upload_part(stream, first_byte, last_byte, file_size)?;
            } // extract the range from the result and update the first_byte and last_byte

            match status {
                UploadStatus::Ok(_) => {
                    // complete the upload
                    return Ok(());
                }
                _ => {
                    return Err(StorageError::Error(format!(
                        "Failed to upload file in chunks: {:?}",
                        status
                    )));
                }
            }

            // check if enum is Ok
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
        fn find(&self, path: &str) -> Result<Vec<String>, StorageError> {
            self.runtime.block_on(async {
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
                    .unwrap_or_else(Vec::new)
                    .iter()
                    .map(|o| o.name.clone())
                    .collect())
            })
        }

        /// Find object information. Runs the same operation as find but returns more information about each object
        ///
        /// # Arguments
        ///
        /// * `path` - The path to list objects from
        ///
        /// # Returns
        ///
        fn find_info(&self, path: &str) -> Result<Vec<FileInfo>, StorageError> {
            let objects = self.runtime.block_on(async {
                let result = self
                    .client
                    .list_objects(&ListObjectsRequest {
                        bucket: self.bucket.clone(),
                        prefix: Some(path.to_string()),
                        ..Default::default()
                    })
                    .await
                    .map_err(|e| StorageError::Error(format!("Unable to list objects: {}", e)))?;
                Ok(result)
            })?;

            Ok(objects
                .items
                .unwrap_or_else(Vec::new)
                .iter()
                .map(|o| FileInfo {
                    name: o.name.clone(),
                    size: o.size.clone(),
                    object_type: o.content_type.clone().unwrap_or_else(|| "".to_string()),
                    created: match o.time_created {
                        Some(last_modified) => last_modified.to_string(),
                        None => "".to_string(),
                    },
                    suffix: o.name.clone().split('.').last().unwrap_or("").to_string(),
                })
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
        fn copy_object(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
            self.runtime.block_on(async {
                self.client
                    .copy_object(
                        &google_cloud_storage::http::objects::copy::CopyObjectRequest {
                            source_bucket: self.bucket.clone(),
                            source_object: src.to_string(),
                            destination_bucket: self.bucket.clone(),
                            destination_object: dest.to_string(),
                            ..Default::default()
                        },
                    )
                    .await
                    .map_err(|e| StorageError::Error(format!("Unable to copy object: {}", e)))?;

                Ok(true)
            })
        }

        /// Copy objects from one bucket to another without deleting the source objects
        ///
        /// # Arguments
        ///
        /// * `src` - The path to the source object
        /// * `dest` - The path to the destination object
        ///
        fn copy_objects(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
            let objects = self.find(src)?;
            let dest = Path::new(dest);
            let src = PathBuf::from(src);

            for obj in objects {
                let file_path = Path::new(obj.as_str());
                let relative_path = file_path.relative_path(&src)?;
                let remote_path = dest.join(relative_path);

                self.copy_object(file_path.to_str().unwrap(), remote_path.to_str().unwrap())?;
            }

            Ok(true)
        }

        /// Delete an object from the storage bucket
        ///
        /// # Arguments
        ///
        /// * `path` - The path to the object in the bucket
        ///
        fn delete_object(&self, path: &str) -> Result<bool, StorageError> {
            let request = DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: path.to_string(),
                ..Default::default()
            };

            self.runtime.block_on(async {
                self.client
                    .delete_object(&request)
                    .await
                    .map_err(|e| StorageError::Error(format!("Unable to delete object: {}", e)))?;

                Ok(true)
            })
        }

        /// Delete an object from the storage bucket
        ///
        /// # Arguments
        ///
        /// * `path` - The path to the object in the bucket
        ///
        fn delete_objects(&self, path: &str) -> Result<bool, StorageError> {
            let request = DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: path.to_string(),
                ..Default::default()
            };

            self.runtime.block_on(async {
                self.client
                    .delete_object(&request)
                    .await
                    .map_err(|e| StorageError::Error(format!("Unable to delete object: {}", e)))?;

                Ok(true)
            })
        }
    }

    impl GoogleStorageClient {
        /// Create a resumable upload session
        ///
        /// # Arguments
        ///
        /// * `path` - The path to the object in the bucket
        ///
        ///
        pub fn create_multipart_upload(
            &self,
            path: &str,
        ) -> Result<GoogleMultipartUploadClient, StorageError> {
            let _filename = path.to_string();

            let metadata = Object {
                name: path.to_string(),
                content_type: Some("application/octet-stream".to_string()),
                ..Default::default()
            };

            self.runtime.block_on(async {
                let result = self
                    .client
                    .prepare_resumable_upload(
                        &UploadObjectRequest {
                            bucket: self.bucket.clone(),
                            ..Default::default()
                        },
                        &UploadType::Multipart(Box::new(metadata)),
                    )
                    .await
                    .map_err(|e| {
                        StorageError::Error(format!("Unable to create resumable session: {}", e))
                    })?;

                Ok(GoogleMultipartUploadClient {
                    client: result,
                    file_completed: false,
                    handle: tokio::runtime::Handle::current(),
                })
            })
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
        pub fn get_object_stream(
            &self,
            rpath: &str,
        ) -> Result<
            impl Stream<Item = Result<bytes::Bytes, google_cloud_storage::http::Error>>,
            StorageError,
        > {
            self.runtime.block_on(async {
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
                    .map_err(|e| {
                        StorageError::Error(format!("Unable to download object: {}", e))
                    })?;
                Ok(result)
            })
        }
    }

    pub struct GCSFSStorageClient {
        client: GoogleStorageClient,
    }

    impl FileSystem<GoogleStorageClient> for GCSFSStorageClient {
        fn client(&self) -> &GoogleStorageClient {
            &self.client
        }
        fn new(bucket: String) -> Self {
            GCSFSStorageClient {
                client: GoogleStorageClient::new(bucket),
            }
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
            let client = GoogleStorageClient::new("bucket".to_string());
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

            let client = GoogleStorageClient::new("bucket".to_string());

            // this will fail because the credentials are fake. We are just testing the client creation
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

            let client = GoogleStorageClient::new("bucket".to_string());
        }
    }
}

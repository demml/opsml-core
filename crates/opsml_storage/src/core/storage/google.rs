#[cfg(feature = "google_storage")]
pub mod google_storage {

    use crate::core::storage::base::{
        get_files, FileInfo, FileSystem, PathExt, StorageClient, StorageSettings,
    };
    use crate::core::utils::error::StorageError;
    use async_trait::async_trait;
    use aws_smithy_types::byte_stream::ByteStream;
    use aws_smithy_types::byte_stream::Length;
    use base64::prelude::*;
    use futures::stream::Stream;
    use futures::StreamExt;
    use google_cloud_auth::credentials::CredentialsFile;
    use google_cloud_storage::client::{Client, ClientConfig};
    use google_cloud_storage::http::objects::delete::DeleteObjectRequest;
    use google_cloud_storage::http::objects::download::Range;
    use google_cloud_storage::http::objects::get::GetObjectRequest;
    use google_cloud_storage::http::objects::list::ListObjectsRequest;
    use google_cloud_storage::http::objects::upload::UploadObjectRequest;
    use google_cloud_storage::http::objects::upload::UploadType;
    use google_cloud_storage::http::objects::Object;
    use google_cloud_storage::http::resumable_upload_client::ChunkSize;
    use google_cloud_storage::http::resumable_upload_client::ResumableUploadClient;
    use google_cloud_storage::http::resumable_upload_client::UploadStatus;
    use google_cloud_storage::sign::SignedURLMethod;
    use google_cloud_storage::sign::SignedURLOptions;
    use pyo3::prelude::*;

    use reqwest_middleware::ClientWithMiddleware;
    use serde_json::Value;
    use std::env;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;

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

    pub struct GoogleMultipartUpload {
        pub client: ResumableUploadClient,
        pub upload_status: UploadStatus,
    }

    impl GoogleMultipartUpload {
        pub async fn new(client: &Client, bucket: &str, path: &str) -> Result<Self, StorageError> {
            let _filename = path.to_string();

            let metadata = Object {
                name: _filename,
                content_type: Some("application/octet-stream".to_string()),
                ..Default::default()
            };

            let result = client
                .prepare_resumable_upload(
                    &UploadObjectRequest {
                        bucket: bucket.to_string(),
                        ..Default::default()
                    },
                    &UploadType::Multipart(Box::new(metadata)),
                )
                .await
                .map_err(|e| {
                    StorageError::Error(format!("Unable to create resumable session: {}", e))
                })?;

            Ok(GoogleMultipartUpload {
                client: result,
                upload_status: UploadStatus::NotStarted,
            })
        }

        pub async fn get_next_chunk(
            &self,
            path: &Path,
            chunk_size: u64,
            chunk_index: u64,
            this_chunk_size: u64,
        ) -> Result<ByteStream, StorageError> {
            let stream = ByteStream::read_from()
                .path(path)
                .offset(chunk_index * chunk_size)
                .length(Length::Exact(this_chunk_size))
                .build()
                .await
                .map_err(|e| StorageError::Error(format!("Failed to get next chunk: {}", e)))?;

            Ok(stream)
        }
        pub async fn upload_part(
            &mut self,
            chunk: ByteStream,
            first_byte: &u64,
            last_byte: &u64,
            file_size: &u64,
        ) -> Result<(), StorageError> {
            let size = ChunkSize::new(*first_byte, *last_byte, Some(*file_size));

            let data = chunk
                .collect()
                .await
                .map_err(|e| StorageError::Error(format!("Unable to collect chunk data: {}", e)))?
                .into_bytes();

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

            self.upload_status = result;

            Ok(())
        }

        pub async fn complete_upload(&mut self) -> Result<(), StorageError> {
            match self.upload_status {
                UploadStatus::Ok(_) => {
                    // complete the upload
                    Ok(())
                }
                _ => Err(StorageError::Error(format!(
                    "Failed to upload file in chunks",
                ))),
            }
        }
    }

    pub struct GoogleStorageClient {
        pub client: Client,
        pub bucket: String,
        pub http: ClientWithMiddleware,
    }

    #[async_trait]
    impl StorageClient for GoogleStorageClient {
        async fn bucket(&self) -> &str {
            &self.bucket
        }
        async fn new(settings: StorageSettings) -> Result<Self, StorageError> {
            let creds = GcpCreds::new().await?;
            // If no credentials, attempt to create a default client pulling from the environment
            let config = if creds.creds.is_none() {
                let config = ClientConfig::default().with_auth().await;

                // if error, use ClientConfig::default().anonymous();
                let config = match config {
                    Ok(config) => config,
                    Err(_) => ClientConfig::default().anonymous(),
                };

                Ok(config)

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

                Ok(config)
            };

            let config = config?;

            let http = config.http.clone().unwrap_or_else(|| {
                reqwest_middleware::ClientBuilder::new(reqwest::Client::default()).build()
            });

            let client = Client::new(config);

            // strip gs:// from the bucket name if exists
            let bucket = settings
                .storage_uri
                .strip_prefix("gs://")
                .unwrap_or(&settings.storage_uri)
                .to_string();

            Ok(GoogleStorageClient {
                client,
                bucket,
                http,
            })
        }

        /// Download a remote object as a stream to a local file
        ///
        /// # Arguments
        ///
        /// * `lpath` - The path to the local file
        /// * `rpath` - The path to the remote file
        ///
        async fn get_object(&self, lpath: &str, rpath: &str) -> Result<(), StorageError> {
            let mut stream = self.get_object_stream(rpath).await?;

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

            while let Some(v) = stream.next().await {
                let chunk = v.map_err(|e| StorageError::Error(format!("Stream error: {}", e)))?;
                file.write_all(&chunk)
                    .map_err(|e| StorageError::Error(format!("Unable to write to file: {}", e)))?;
            }

            Ok(())
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
        async fn generate_presigned_url(
            &self,
            path: &str,
            expiration: u64,
        ) -> Result<String, StorageError> {
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
        async fn find(&self, path: &str) -> Result<Vec<String>, StorageError> {
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
        }

        /// Find object information. Runs the same operation as find but returns more information about each object
        ///
        /// # Arguments
        ///
        /// * `path` - The path to list objects from
        ///
        /// # Returns
        ///
        async fn find_info(&self, path: &str) -> Result<Vec<FileInfo>, StorageError> {
            let result = self
                .client
                .list_objects(&ListObjectsRequest {
                    bucket: self.bucket.clone(),
                    prefix: Some(path.to_string()),
                    ..Default::default()
                })
                .await
                .map_err(|e| StorageError::Error(format!("Unable to list objects: {}", e)))?;

            Ok(result
                .items
                .unwrap_or_else(Vec::new)
                .iter()
                .map(|o| FileInfo {
                    name: o.name.clone(),
                    size: o.size,
                    object_type: o.content_type.clone().unwrap_or_default(),
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
        async fn copy_object(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
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
        }

        /// Copy objects from one bucket to another without deleting the source objects
        ///
        /// # Arguments
        ///
        /// * `src` - The path to the source object
        /// * `dest` - The path to the destination object
        ///
        async fn copy_objects(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
            let objects = self.find(src).await?;
            let dest = Path::new(dest);
            let src = PathBuf::from(src);

            for obj in objects {
                let file_path = Path::new(obj.as_str());
                let relative_path = file_path.relative_path(&src)?;
                let remote_path = dest.join(relative_path);

                self.copy_object(file_path.to_str().unwrap(), remote_path.to_str().unwrap())
                    .await?;
            }

            Ok(true)
        }

        /// Delete an object from the storage bucket
        ///
        /// # Arguments
        ///
        /// * `path` - The path to the object in the bucket
        ///
        async fn delete_object(&self, path: &str) -> Result<bool, StorageError> {
            let request = DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: path.to_string(),
                ..Default::default()
            };

            self.client
                .delete_object(&request)
                .await
                .map_err(|e| StorageError::Error(format!("Unable to delete object: {}", e)))?;

            Ok(true)
        }

        /// Delete an object from the storage bucket
        ///
        /// # Arguments
        ///
        /// * `path` - The path to the object in the bucket
        ///
        async fn delete_objects(&self, path: &str) -> Result<bool, StorageError> {
            let objects = self.find(path).await?;

            for obj in objects {
                self.delete_object(obj.as_str()).await?;
            }

            Ok(true)
        }
    }

    impl GoogleStorageClient {
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
        ) -> Result<
            impl Stream<Item = Result<bytes::Bytes, google_cloud_storage::http::Error>>,
            StorageError,
        > {
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
            Ok(result)
        }

        pub async fn create_multipart_upload(&self, path: &str) -> Result<String, StorageError> {
            let _filename = path.to_string();

            let metadata = Object {
                name: _filename.clone(),
                content_type: Some("application/octet-stream".to_string()),
                ..Default::default()
            };

            let result = self
                .client
                .prepare_resumable_upload(
                    &UploadObjectRequest {
                        bucket: self.bucket.to_string(),
                        ..Default::default()
                    },
                    &UploadType::Multipart(Box::new(metadata)),
                )
                .await
                .map_err(|e| {
                    StorageError::Error(format!("Unable to create resumable session: {}", e))
                })?;

            Ok(result.url().to_string())
        }

        pub async fn get_uploader(
            &self,
            path: &str,
        ) -> Result<GoogleMultipartUpload, StorageError> {
            let client = GoogleMultipartUpload::new(&self.client, &self.bucket, path).await?;
            Ok(client)
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
        async fn upload_file_in_chunks(
            &self,
            lpath: &Path,
            uploader: &mut GoogleMultipartUpload,
        ) -> Result<(), StorageError> {
            let file = File::open(lpath)
                .map_err(|e| StorageError::Error(format!("Failed to open file: {}", e)))?;

            let metadata = file
                .metadata()
                .map_err(|e| StorageError::Error(format!("Failed to get file metadata: {}", e)))?;

            let file_size = metadata.len();
            let chunk_size = std::cmp::min(file_size, 1024 * 1024 * 5);

            // calculate the number of parts
            let mut chunk_count = (file_size / chunk_size) + 1;
            let mut size_of_last_chunk = file_size % chunk_size;

            // if the last chunk is empty, reduce the number of parts
            if size_of_last_chunk == 0 {
                size_of_last_chunk = chunk_size;
                chunk_count -= 1;
            }

            for chunk_index in 0..chunk_count {
                let this_chunk = if chunk_count - 1 == chunk_index {
                    size_of_last_chunk
                } else {
                    chunk_size
                };

                let first_byte = chunk_index * chunk_size;
                let last_byte = first_byte + this_chunk - 1;

                let chunk = uploader
                    .get_next_chunk(lpath, chunk_size, chunk_index, this_chunk)
                    .await?;

                uploader
                    .upload_part(chunk, &first_byte, &last_byte, &file_size)
                    .await?;
            } // extract the range from the result and update the first_byte and last_byte

            match uploader.upload_status {
                UploadStatus::Ok(_) => {
                    // complete the upload
                    Ok(())
                }
                _ => Err(StorageError::Error(format!(
                    "Failed to upload file in chunks"
                ))),
            }

            // check if enum is Ok
        }
    }

    pub struct GCSFSStorageClient {
        client: GoogleStorageClient,
    }

    #[async_trait]
    impl FileSystem<GoogleStorageClient> for GCSFSStorageClient {
        fn name(&self) -> &str {
            "GCSFSStorageClient"
        }
        fn client(&self) -> &GoogleStorageClient {
            &self.client
        }
        async fn new(settings: StorageSettings) -> Self {
            GCSFSStorageClient {
                client: GoogleStorageClient::new(settings).await.unwrap(),
            }
        }
    }

    impl GCSFSStorageClient {
        pub async fn put(
            &self,
            lpath: &Path,
            rpath: &Path,
            recursive: bool,
        ) -> Result<(), StorageError> {
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

                    let mut uploader = self
                        .client()
                        .get_uploader(remote_path.to_str().unwrap())
                        .await?;

                    self.client()
                        .upload_file_in_chunks(&stripped_file_path, &mut uploader)
                        .await?;
                }

                Ok(())
            } else {
                let mut uploader = self
                    .client()
                    .get_uploader(stripped_rpath.to_str().unwrap())
                    .await?;

                self.client()
                    .upload_file_in_chunks(&stripped_lpath, &mut uploader)
                    .await?;
                Ok(())
            }
        }
    }

    #[pyclass]
    pub struct PyGCSFSStorageClient {
        client: GoogleStorageClient,
        runtime: tokio::runtime::Runtime,
    }

    #[pymethods]
    impl PyGCSFSStorageClient {
        #[new]
        fn new(settings: StorageSettings) -> Self {
            let rt = tokio::runtime::Runtime::new().unwrap();

            let client = rt.block_on(async { GoogleStorageClient::new(settings).await.unwrap() });

            Self {
                client,
                runtime: rt,
            }
        }

        fn find_info(&self, path: PathBuf) -> Result<Vec<FileInfo>, StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);

            self.runtime
                .block_on(async { self.client.find_info(stripped_path.to_str().unwrap()).await })
        }

        #[pyo3(signature = (path=PathBuf::new()))]
        fn find(&self, path: PathBuf) -> Result<Vec<String>, StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);

            self.runtime
                .block_on(async { self.client.find(stripped_path.to_str().unwrap()).await })
        }

        #[pyo3(signature = (lpath, rpath, recursive = false))]
        fn get(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError> {
            // strip the paths
            let stripped_rpath = rpath.strip_path(&self.client.bucket);
            let stripped_lpath = lpath.strip_path(&self.client.bucket);

            self.runtime.block_on(async {
                if recursive {
                    let stripped_lpath_clone = stripped_lpath.clone();

                    // list all objects in the path

                    let objects = self.client.find(stripped_rpath.to_str().unwrap()).await?;

                    // iterate over each object and get it
                    for obj in objects {
                        let file_path = Path::new(obj.as_str());
                        let stripped_path = file_path.strip_path(&self.client.bucket);
                        let relative_path = file_path.relative_path(&stripped_rpath)?;
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
                        .get_object(
                            stripped_lpath.to_str().unwrap(),
                            stripped_rpath.to_str().unwrap(),
                        )
                        .await?;
                }

                Ok(())
            })
        }

        #[pyo3(signature = (lpath, rpath, recursive = false))]
        fn put(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError> {
            let stripped_lpath = lpath.strip_path(&self.client.bucket);
            let stripped_rpath = rpath.strip_path(&self.client.bucket);

            self.runtime.block_on(async {
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
                        let stripped_file_path = file.strip_path(&self.client.bucket);

                        let relative_path = file.relative_path(&stripped_lpath_clone)?;
                        let remote_path = stripped_rpath_clone.join(relative_path);

                        let mut uploader = self
                            .client
                            .get_uploader(remote_path.to_str().unwrap())
                            .await?;

                        self.client
                            .upload_file_in_chunks(&stripped_file_path, &mut uploader)
                            .await?;
                    }

                    Ok(())
                } else {
                    let mut uploader = self
                        .client
                        .get_uploader(stripped_rpath.to_str().unwrap())
                        .await?;

                    self.client
                        .upload_file_in_chunks(&stripped_lpath, &mut uploader)
                        .await?;
                    Ok(())
                }
            })
        }

        #[pyo3(signature = (src, dest, recursive = false))]
        fn copy(&self, src: PathBuf, dest: PathBuf, recursive: bool) -> Result<(), StorageError> {
            let stripped_src = src.strip_path(&self.client.bucket);
            let stripped_dest = dest.strip_path(&self.client.bucket);

            self.runtime.block_on(async {
                if recursive {
                    self.client
                        .copy_objects(
                            stripped_src.to_str().unwrap(),
                            stripped_dest.to_str().unwrap(),
                        )
                        .await?;
                } else {
                    self.client
                        .copy_object(
                            stripped_src.to_str().unwrap(),
                            stripped_dest.to_str().unwrap(),
                        )
                        .await?;
                }

                Ok(())
            })
        }

        #[pyo3(signature = (path, recursive = false))]
        fn rm(&self, path: PathBuf, recursive: bool) -> Result<(), StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);

            self.runtime.block_on(async {
                if recursive {
                    self.client
                        .delete_objects(stripped_path.to_str().unwrap())
                        .await?;
                } else {
                    self.client
                        .delete_object(stripped_path.to_str().unwrap())
                        .await?;
                }

                Ok(())
            })
        }

        fn exists(&self, path: PathBuf) -> Result<bool, StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);

            let objects = self
                .runtime
                .block_on(async { self.client.find(stripped_path.to_str().unwrap()).await })?;

            Ok(!objects.is_empty())
        }

        #[pyo3(signature = (path, expiration = 600))]
        fn generate_presigned_url(
            &self,
            path: PathBuf,
            expiration: u64,
        ) -> Result<String, StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);

            self.runtime.block_on(async {
                self.client
                    .generate_presigned_url(stripped_path.to_str().unwrap(), expiration)
                    .await
            })
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::io::Write;
        use tokio;

        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};

        const CHUNK_SIZE: u64 = 1024 * 256;

        pub fn get_settings() -> StorageSettings {
            let bucket = std::env::var("CLOUD_BUCKET_NAME")
                .unwrap_or_else(|_| "opsml-storage-integration".to_string());

            StorageSettings {
                storage_uri: bucket,
                ..Default::default()
            }
        }

        pub fn create_file(name: &str, chunk_size: &u64) {
            let mut file = File::create(name).expect("Could not create sample file.");

            while file.metadata().unwrap().len() <= chunk_size * 2 {
                let rand_string: String = thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(256)
                    .map(char::from)
                    .collect();
                let return_string: String = "\n".to_string();
                file.write_all(rand_string.as_ref())
                    .expect("Error writing to file.");
                file.write_all(return_string.as_ref())
                    .expect("Error writing to file.");
            }
        }

        pub fn create_nested_data(chunk_size: &u64) -> String {
            let rand_name = uuid::Uuid::new_v4().to_string();

            // create a temporary directory
            let dir_name = format!("temp_test_dir_{}", &rand_name);
            let dir = Path::new(&dir_name);

            if !dir.exists() {
                std::fs::create_dir_all(dir).unwrap();
            }
            // random file name with uuid
            let key = format!("{}/temp_test_file_{}.txt", &dir_name, &rand_name);
            create_file(&key, chunk_size);

            // created nested directories
            let dir = Path::new(&dir_name);
            let nested_dir = dir.join("nested_dir");
            let nested_dir_path = nested_dir.to_str().unwrap();

            if !nested_dir.exists() {
                std::fs::create_dir_all(nested_dir.clone()).unwrap();
            }

            // random file name with uuid
            let key = format!("{}/temp_test_file_{}.txt", &nested_dir_path, &rand_name);
            create_file(&key, chunk_size);

            dir_name
        }

        fn create_single_file(chunk_size: &u64) -> String {
            let rand_name = uuid::Uuid::new_v4().to_string();

            // create a temporary directory
            let dir_name = format!("temp_test_dir_{}", &rand_name);
            let dir = Path::new(&dir_name);

            if !dir.exists() {
                std::fs::create_dir_all(dir).unwrap();
            }

            // random file name with uuid
            let key = format!("{}/temp_test_file_{}.txt", &dir_name, &rand_name);
            create_file(&key, chunk_size);

            key
        }

        #[tokio::test]
        async fn test_google_creds_new() {
            let creds = GcpCreds::new().await;
            assert!(creds.is_ok());
        }

        #[test]
        fn test_google_client_new() {
            let settings = get_settings();
            let _client = GoogleStorageClient::new(settings);
        }

        #[tokio::test]
        async fn test_google_storage_client_get_object() {
            let settings = get_settings();
            let client = GoogleStorageClient::new(settings).await.unwrap();

            // should fail since there are no suffixes
            let result = client.get_object("local_path", "remote_path").await;
            assert!(result.is_err()); // Assuming the object does not exist
        }

        #[tokio::test]
        async fn test_google_storage_client_put() {
            let settings = get_settings();
            let client = GCSFSStorageClient::new(settings).await;

            //
            let dirname = create_nested_data(&CHUNK_SIZE);

            let lpath = Path::new(&dirname);
            let rpath = Path::new(&dirname);

            // put the file
            client.put(lpath, rpath, true).await.unwrap();

            // check if the file exists
            let exists = client.exists(rpath).await.unwrap();
            assert!(exists);

            // list all files
            let files = client.find(rpath).await.unwrap();
            assert_eq!(files.len(), 2);

            // list files with info
            let files = client.find_info(rpath).await.unwrap();
            assert_eq!(files.len(), 2);

            // download the files
            let new_path = uuid::Uuid::new_v4().to_string();
            let new_path = Path::new(&new_path);

            client.get(new_path, rpath, true).await.unwrap();

            // check if the files are the same
            let files = get_files(rpath).unwrap();
            let new_files = get_files(new_path).unwrap();

            assert_eq!(files.len(), new_files.len());

            // copy the files
            // create a new path
            let copy_path = uuid::Uuid::new_v4().to_string();
            let copy_path = Path::new(&copy_path);
            client.copy(rpath, copy_path, true).await.unwrap();
            let files = client.find(copy_path).await.unwrap();
            assert_eq!(files.len(), 2);

            // cleanup
            std::fs::remove_dir_all(&dirname).unwrap();
            std::fs::remove_dir_all(new_path).unwrap();

            client.rm(rpath, true).await.unwrap();
            client.rm(copy_path, true).await.unwrap();

            // check if the file exists
            let exists = client.exists(rpath).await.unwrap();
            assert!(!exists);
        }

        #[tokio::test]
        async fn test_google_storage_client_generate_presigned_url() {
            let settings = get_settings();
            let client = GCSFSStorageClient::new(settings).await;

            // create file
            let key = create_single_file(&CHUNK_SIZE);
            let path = Path::new(&key);

            // put the file
            client.put(path, path, false).await.unwrap();

            // generate presigned url
            let url = client.generate_presigned_url(path, 3600).await.unwrap();
            assert!(!url.is_empty());

            // cleanup
            client.rm(path, false).await.unwrap();
            std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
        }

        #[tokio::test]
        async fn test_google_large_file_upload() {
            let settings = get_settings();
            let client = GCSFSStorageClient::new(settings).await;

            // create file
            let chunk_size = 1024 * 1024 * 5; // 5MB
            let key = create_single_file(&chunk_size);
            let path = Path::new(&key);

            // put the file
            client.put(path, path, false).await.unwrap();

            // get the file info
            let info = client.find_info(path).await.unwrap();
            assert_eq!(info.len(), 1);

            // get item and assert it's at least the size of the file
            let item = info.first().unwrap();
            assert!(item.size >= 1024 * 1024 * 10);

            // cleanup
            client.rm(path, false).await.unwrap();
            std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
        }
    }
}

#[cfg(feature = "google_storage")]
pub mod google_storage {
    use crate::core::storage::base::ByteIterator;
    use crate::core::utils::error::StorageError;
    use base64::prelude::*;
    use futures::stream::Stream;
    use futures::StreamExt;
    use futures::TryStream;
    use futures::TryStreamExt;
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
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;
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

    #[derive(Clone)]
    pub struct GoogleResumableUploadClient {
        pub client: ResumableUploadClient,
        first_byte: u64,
        last_byte: u64,
        total_size: u64,
        chunk_size: u64,
        file_completed: bool,
    }

    impl GoogleResumableUploadClient {
        pub async fn upload_multiple_chunks(
            &mut self,
            chunk: bytes::Bytes,
        ) -> Result<Option<String>, StorageError> {
            let size = ChunkSize::new(self.first_byte, 63304, Some(self.total_size));

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
                UploadStatus::Ok(object) => {
                    self.file_completed = true;
                    return Ok(Some(object.name));
                }
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

                Ok(Client::new(config))

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
                Ok(client)
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

            // chunk the stream and return a stream of bytes

            // return stream of bytes
            Ok(result)
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
                .unwrap_or_else(Vec::new)
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
            let _filename = path.to_string();

            let metadata = Object {
                name: path.to_string(),
                content_type: Some("application/octet-stream".to_string()),
                ..Default::default()
            };

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

            Ok(GoogleResumableUploadClient {
                client: result,
                first_byte: 0,
                last_byte: chunk_size,
                total_size,
                chunk_size,
                file_completed: false,
            })
        }

        /// Upload file in chunks. This method will take a file path, open the file, read it in chunks and upload each chunk to the object
        ///
        /// # Arguments
        ///
        /// * `resumable_client` - GoogleResumableUploadClient
        /// * `lpath` - The path to the file to upload
        ///
        /// # Returns
        ///
        /// A Result with the object name if successful
        ///    
        pub async fn upload_file_in_chunks(
            &self,
            resumable_client: &mut GoogleResumableUploadClient,
            lpath: &Path,
        ) -> Result<bool, StorageError> {
            // get chunk size
            let chunk_size = resumable_client.chunk_size as usize;

            // open file
            let file = std::fs::File::open(lpath).map_err(|e| {
                StorageError::Error(format!("Unable to open file for reading: {}", e))
            })?;

            // create a buffered reader
            let mut reader = BufReader::with_capacity(chunk_size, file);
            let mut first_byte = 0;

            loop {
                let buffer = &reader.fill_buf().map_err(|e| {
                    StorageError::Error(format!("Unable to read buffer from file: {}", e))
                })?;

                let buffer_size = buffer.len();
                if buffer_size == 0 {
                    break;
                }

                let last_byte = first_byte + buffer_size as u64 - 1;
                let size = ChunkSize::new(first_byte, last_byte, Some(resumable_client.total_size));

                resumable_client
                    .client
                    .upload_multiple_chunk(bytes::Bytes::from(buffer.to_vec()), &size)
                    .await
                    .unwrap();

                first_byte = last_byte + 1;

                // consume buffer
                reader.consume(buffer_size);
            }

            Ok(resumable_client.file_completed)
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
        pub async fn copy_object(&self, src: &str, dest: &str) -> Result<String, StorageError> {
            let result = self
                .client
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

            Ok(result.name)
        }

        /// Delete an object from the storage bucket
        ///
        /// # Arguments
        ///
        /// * `path` - The path to the object in the bucket
        ///
        pub async fn delete_object(&self, path: &str) -> Result<(), StorageError> {
            let request = DeleteObjectRequest {
                bucket: self.bucket.clone(),
                object: path.to_string(),
                ..Default::default()
            };

            self.client
                .delete_object(&request)
                .await
                .map_err(|e| StorageError::Error(format!("Unable to delete object: {}", e)))?;

            Ok(())
        }

        /// Download a remote object as a stream to a local file
        ///
        /// # Arguments
        ///
        /// * `lpath` - The path to the local file
        /// * `rpath` - The path to the remote file
        ///
        pub async fn get_object(&self, lpath: &str, rpath: &str) -> Result<(), StorageError> {
            let mut stream = self
                .get_object_stream(rpath)
                .await?
                .map_err(|e| StorageError::Error(format!("Unable to get object stream: {}", e)));

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

            // iterate over the stream and write to the file
            while let Some(v) = stream.next().await {
                let chunk = v.map_err(|e| StorageError::Error(format!("Stream error: {}", e)))?;
                file.write_all(&chunk)
                    .map_err(|e| StorageError::Error(format!("Unable to write to file: {}", e)))?;
            }

            Ok(())
        }

        pub async fn generate_presigned_url(
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

        pub fn find(&self, path: PathBuf) -> PyResult<Vec<String>> {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async {
                // remove bucket from path if exists
                let stripped_path = path
                    .strip_prefix(&self.client.bucket)
                    .unwrap_or(&path)
                    .to_str()
                    .unwrap();

                self.client
                    .find(stripped_path)
                    .await
                    .map_err(|e| StorageError::Error(format!("Unable to list objects: {}", e)))
            });

            Ok(
                result
                    .map_err(|e| StorageError::Error(format!("Unable to list objects: {}", e)))?,
            )
        }

        pub fn iterfile(&self, path: PathBuf) -> PyResult<ByteIterator> {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async {
                // remove bucket from path if exists
                let stripped_path = path
                    .strip_prefix(&self.client.bucket)
                    .unwrap_or(&path)
                    .to_str()
                    .unwrap();

                self.client.get_object_stream(stripped_path).await
            });

            let stream = result
                .unwrap()
                .map_err(|e| StorageError::Error(format!("Stream error: {}", e)));

            Ok(ByteIterator {
                stream: Box::new(stream),
                runtime: rt,
            })
        }

        /// Upload a file to the storage bucket
        ///
        /// # Arguments
        ///
        /// * `lpath` - The path to the local file
        /// * `rpath` - The path to the remote file
        ///
        pub fn put(&self, lpath: PathBuf, rpath: PathBuf) -> PyResult<()> {
            // start a new runtime
            let rt = Runtime::new().unwrap();

            // remove bucket from path if exists
            let (stripped_lpath, stripped_rpath) = self.strip_paths(lpath, rpath);

            // set chunk size (8 mib)
            let chunk_size = 1024 * 1024 * 8;

            let result: Result<(), StorageError> = rt.block_on(async {
                // check if stripped path is a directory
                if stripped_lpath.is_dir() {
                    // get all files in the directory
                    let files: Vec<_> = walkdir::WalkDir::new(stripped_lpath.clone())
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_file())
                        .map(|e| e.path().to_path_buf())
                        .collect();

                    // create a vector to hold all the upload tasks
                    let mut upload_tasks = Vec::new();

                    // iterate over the files and upload them
                    for file in files {
                        // clone the client
                        let client = self.client.clone();
                        let stripped_lpath_clone = stripped_lpath.clone();
                        let stripped_rpath = stripped_rpath.clone();

                        // create a new task for each file upload
                        let upload_task = tokio::spawn(async move {
                            let stripped_path = file.strip_prefix(&client.bucket).unwrap_or(&file);

                            // get file size for stripped path to pass to resumable upload session
                            let metadata = std::fs::metadata(stripped_path).unwrap();

                            // get the relative path of the file to the stripped lpath
                            let relative_path = file.strip_prefix(stripped_lpath_clone).unwrap();

                            // add the relative path to the stripped rpath
                            let remote_path = Path::new(&stripped_rpath).join(relative_path);

                            //create a resumable upload session
                            let resumable_upload = client
                                .create_resumable_upload_session(
                                    remote_path.to_str().unwrap(), // create file at remote path
                                    std::cmp::min(chunk_size, metadata.len()),
                                    metadata.len(), // total size
                                )
                                .await
                                .map_err(|e| {
                                    StorageError::Error(format!(
                                        "Unable to create resumable upload session: {}",
                                        e
                                    ))
                                });

                            match resumable_upload {
                                Ok(mut resumable_client) => {
                                    client
                                        .upload_file_in_chunks(&mut resumable_client, stripped_path)
                                        .await?;
                                }
                                Err(e) => {
                                    return Err(e);
                                }
                            }

                            Ok::<(), StorageError>(())
                        });

                        // add the task to the vector
                        upload_tasks.push(upload_task);
                    }

                    // wait for all tasks to complete
                    futures::future::join_all(upload_tasks).await;
                } else {
                    let metadata = std::fs::metadata(&stripped_lpath).unwrap();

                    //create a resumable upload session
                    let mut resumable_client = self
                        .client
                        .create_resumable_upload_session(
                            stripped_rpath.to_str().unwrap(),
                            std::cmp::min(chunk_size, metadata.len()),
                            metadata.len(),
                        )
                        .await
                        .map_err(|e| {
                            StorageError::Error(format!(
                                "Unable to create resumable upload session: {}",
                                e
                            ))
                        })?;

                    self.client
                        .upload_file_in_chunks(&mut resumable_client, &stripped_lpath)
                        .await?;
                }
                Ok(())
            });

            result.map_err(|e| -> pyo3::PyErr {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Unable to upload file: {}", e))
            })
        }

        fn strip_paths(&self, src: PathBuf, dest: PathBuf) -> (PathBuf, PathBuf) {
            // remove bucket from path if exists
            let stripped_src = src
                .strip_prefix(&self.client.bucket)
                .unwrap_or(&src)
                .to_path_buf();

            // remove bucket from path if exists
            let stripped_dest = dest
                .strip_prefix(&self.client.bucket)
                .unwrap_or(&dest)
                .to_path_buf();

            (stripped_src, stripped_dest)
        }

        pub fn copy(&self, src: PathBuf, dest: PathBuf, recursive: bool) -> PyResult<()> {
            let rt = Runtime::new().unwrap();

            let result = rt.block_on(async {
                // remove bucket from path if exists
                let (stripped_src, stripped_dest) = self.strip_paths(src, dest);

                if recursive {
                    let files = self.client.find(stripped_src.to_str().unwrap()).await?;

                    let mut upload_tasks = Vec::new();

                    for file in files {
                        // clone the client

                        let stripped_lpath_clone = stripped_src.clone();
                        let stripped_rpath = stripped_dest.clone();

                        let client = self.client.clone();

                        let upload_task = tokio::spawn(async move {
                            let file_path = Path::new(&file);

                            let src_path =
                                file_path.strip_prefix(&client.bucket).unwrap_or(file_path);

                            // get the relative path of the file to the stripped lpath
                            let relative_path =
                                file_path.strip_prefix(stripped_lpath_clone).unwrap();

                            let dest_path = stripped_rpath.join(relative_path);

                            client
                                .copy_object(
                                    src_path.to_str().unwrap(),
                                    dest_path.to_str().unwrap(),
                                )
                                .await?;

                            Ok::<(), StorageError>(())
                        });

                        upload_tasks.push(upload_task);
                    }

                    futures::future::join_all(upload_tasks).await;
                } else {
                    let copied = self
                        .client
                        .copy_object(
                            stripped_src.to_str().unwrap(),
                            stripped_dest.to_str().unwrap(),
                        )
                        .await;

                    match copied {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(StorageError::Error(format!(
                                "Unable to copy object: {}",
                                e
                            )));
                        }
                    }
                }

                Ok(())
            });

            result.map_err(|e| -> pyo3::PyErr {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Unable to copy file: {}", e))
            })
        }

        pub fn rm(&self, path: PathBuf, recursive: bool) -> PyResult<()> {
            let rt = Runtime::new().unwrap();

            let result: Result<(), StorageError> = rt.block_on(async {
                // remove bucket from path if exists
                let stripped_path = path
                    .strip_prefix(&self.client.bucket)
                    .unwrap_or(&path)
                    .to_str()
                    .unwrap();

                if recursive {
                    let files = self.client.find(stripped_path).await?;

                    let mut upload_tasks = Vec::new();

                    for file in files {
                        // clone the client
                        let client = self.client.clone();

                        let upload_task = tokio::spawn(async move {
                            client.delete_object(&file).await?;
                            Ok::<(), StorageError>(())
                        });

                        upload_tasks.push(upload_task);
                    }

                    let results = futures::future::join_all(upload_tasks).await;
                    for result in results {
                        result.map_err(|e| StorageError::Error(format!("Join error: {}", e)))??;
                    }
                } else {
                    self.client.delete_object(stripped_path).await?;
                }

                Ok(())
            });

            result.map_err(|e| -> pyo3::PyErr {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Unable to delete file: {}", e))
            })
        }

        pub fn exists(&self, path: PathBuf) -> PyResult<bool> {
            let rt = Runtime::new().unwrap();
            let result: Result<bool, StorageError> = rt.block_on(async {
                // remove bucket from path if exists
                let stripped_path = path
                    .strip_prefix(&self.client.bucket)
                    .unwrap_or(&path)
                    .to_str()
                    .unwrap();

                let files = self.client.find(stripped_path).await?;

                Ok(!files.is_empty())
            });

            result.map_err(|e| -> pyo3::PyErr {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Unable to check if file exists: {}",
                    e
                ))
            })
        }

        pub fn get(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> PyResult<()> {
            let rt = Runtime::new().unwrap();

            let result = rt.block_on(async {
                // remove bucket from path if exists
                let (stripped_lpath, stripped_rpath) = self.strip_paths(lpath, rpath);

                if recursive {
                    let files = self.client.find(stripped_rpath.to_str().unwrap()).await?;

                    let mut upload_tasks = Vec::new();

                    for file in files {
                        // clone the client
                        let client = self.client.clone();

                        let stripped_lpath_clone = stripped_lpath.clone();
                        let stripped_rpath = stripped_rpath.clone();

                        let upload_task = tokio::spawn(async move {
                            let file_path = Path::new(&file);
                            let stripped_path =
                                file_path.strip_prefix(&client.bucket).unwrap_or(file_path);

                            // get the relative path of the file to the stripped lpath
                            let relative_path = file_path.strip_prefix(stripped_rpath).unwrap();

                            let local_path = stripped_lpath_clone.join(relative_path);

                            client
                                .get_object(
                                    local_path.to_str().unwrap(),
                                    stripped_path.to_str().unwrap(),
                                )
                                .await?;

                            Ok::<(), StorageError>(())
                        });

                        upload_tasks.push(upload_task);
                    }

                    futures::future::join_all(upload_tasks).await;
                } else {
                    let copied = self
                        .client
                        .get_object(
                            stripped_lpath.to_str().unwrap(),
                            stripped_rpath.to_str().unwrap(),
                        )
                        .await;

                    match copied {
                        Ok(_) => {}
                        Err(e) => {
                            return Err(StorageError::Error(format!(
                                "Unable to get object: {}",
                                e
                            )));
                        }
                    }
                }

                Ok(())
            });

            result.map_err(|e| -> pyo3::PyErr {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Unable to get file: {}", e))
            })
        }

        #[pyo3(signature = (path, expiration=600))]
        pub fn generate_presigned_url(&self, path: PathBuf, expiration: u64) -> PyResult<String> {
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(async {
                // remove bucket from path if exists
                let stripped_path = path
                    .strip_prefix(&self.client.bucket)
                    .unwrap_or(&path)
                    .to_str()
                    .unwrap();

                self.client
                    .generate_presigned_url(stripped_path, expiration)
                    .await
            });

            result.map_err(|e| -> pyo3::PyErr {
                pyo3::exceptions::PyRuntimeError::new_err(format!(
                    "Unable to generate presigned url: {}",
                    e
                ))
            })
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

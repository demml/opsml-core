#[cfg(feature = "aws_storage")]
pub mod aws_storage {
    use crate::core::storage::base::{get_files, FileInfo, FileSystem, PathExt, StorageClient};
    use opsml_settings::config::{OpsmlStorageSettings, StorageType};

    use crate::core::utils::error::StorageError;
    use async_trait::async_trait;
    use aws_config::BehaviorVersion;
    use aws_config::SdkConfig;

    use aws_sdk_s3::operation::get_object::GetObjectOutput;
    use aws_sdk_s3::presigning::PresigningConfig;
    use aws_sdk_s3::primitives::ByteStream;
    use aws_sdk_s3::primitives::Length;
    use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
    use aws_sdk_s3::Client;
    use pyo3::prelude::*;
    use reqwest::Client as HttpClient;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;
    use std::str;
    use std::time::Duration;

    const MAX_CHUNKS: u64 = 10000;

    /// Notes:
    /// For general compatibility with the Pyo3, Rust and generics, we need to define structs with sync in mind.
    /// Thus, some structs and functions will need to spawn a new runtime to run async functions from a sync context.
    /// This is handled at the 3rd-party abstraction level, so the user does not need to worry about it.

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

    pub struct AWSMulitPartUpload {
        pub client: Client,
        pub bucket: String,
        pub path: String,
        pub upload_id: String,
        upload_parts: Vec<aws_sdk_s3::types::CompletedPart>,
    }

    impl AWSMulitPartUpload {
        pub async fn new(bucket: &str, path: &str, upload_id: &str) -> Result<Self, StorageError> {
            // create a resuable runtime for the multipart upload

            let creds = AWSCreds::new().await?;
            let client = Client::new(&creds.config);

            let _bucket = bucket.to_string();
            let _path = path.to_string();

            Ok(Self {
                client,
                bucket: _bucket,
                path: _path,
                upload_id: upload_id.to_string(),
                upload_parts: Vec::new(),
            })
        }

        pub async fn upload_part_with_presigned_url(
            &mut self,
            part_number: &i32,
            body: ByteStream,
            presigned_url: &str,
        ) -> Result<bool, StorageError> {
            // collect the ByteStream
            let body = body
                .collect()
                .await
                .map_err(|e| StorageError::Error(format!("Failed to collect ByteStream: {}", e)))?;

            // convert to bytes::Bytes

            let http_client = HttpClient::new();
            let response = http_client
                .put(presigned_url)
                .body(body.into_bytes())
                .send()
                .await
                .map_err(|e| StorageError::Error(format!("Failed to upload part: {}", e)))?;

            if response.status().is_success() {
                self.upload_parts.push(
                    CompletedPart::builder()
                        .e_tag(
                            response
                                .headers()
                                .get("ETag")
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string(),
                        )
                        .part_number(*part_number)
                        .build(),
                );
                Ok(true)
            } else {
                Err(StorageError::Error(format!(
                    "Failed to upload part: {}",
                    response.status()
                )))
            }
        }

        pub async fn upload_part(
            &mut self,
            part_number: i32,
            body: ByteStream,
        ) -> Result<bool, StorageError> {
            let response = self
                .client
                .upload_part()
                .bucket(&self.bucket)
                .key(&self.path)
                .upload_id(&self.upload_id)
                .body(body)
                .part_number(part_number)
                .send()
                .await
                .map_err(|e| StorageError::Error(format!("Failed to upload part: {}", e)))?;

            self.upload_parts.push(
                CompletedPart::builder()
                    .e_tag(response.e_tag.unwrap_or_default())
                    .part_number(part_number)
                    .build(),
            );

            Ok(true)
        }

        pub async fn complete_upload(&self) -> Result<(), StorageError> {
            let completed_multipart_upload: CompletedMultipartUpload =
                CompletedMultipartUpload::builder()
                    .set_parts(Some(self.upload_parts.clone()))
                    .build();

            let _complete_multipart_upload_res = self
                .client
                .complete_multipart_upload()
                .bucket(&self.bucket)
                .key(&self.path)
                .multipart_upload(completed_multipart_upload)
                .upload_id(&self.upload_id)
                .send()
                .await
                .map_err(|e| {
                    StorageError::Error(format!("Failed to complete multipart upload: {}", e))
                })?;

            Ok(())
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
    }

    pub struct AWSStorageClient {
        pub client: Client,
        pub bucket: String,
    }

    #[async_trait]
    impl StorageClient for AWSStorageClient {
        fn storage_type(&self) -> StorageType {
            StorageType::AWS
        }
        async fn bucket(&self) -> &str {
            &self.bucket
        }
        async fn new(settings: &OpsmlStorageSettings) -> Result<Self, StorageError> {
            // create a resuable runtime for client

            let creds = AWSCreds::new().await?;
            let client = Client::new(&creds.config);

            let bucket = settings
                .storage_uri
                .strip_prefix("s3://")
                .unwrap_or(&settings.storage_uri)
                .to_string();

            Ok(Self { client, bucket })
        }

        async fn get_object(&self, lpath: &str, rpath: &str) -> Result<(), StorageError> {
            // check if lpath and rpath have suffixes
            let lpath = Path::new(lpath);
            let rpath = Path::new(rpath);

            // fail if lpath and rpath have no suffixes
            if lpath.extension().is_none() || rpath.extension().is_none() {
                return Err(StorageError::Error(
                    "Local and remote paths must have suffixes".to_string(),
                ));
            }

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

            // get stream
            let mut response = self.get_object_stream(rpath.to_str().unwrap()).await?;

            // iterate over the stream and write to the file
            while let Some(v) = response.body.next().await {
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
            let expires_in = std::time::Duration::from_secs(expiration);

            let uri = self
                .client
                .get_object()
                .bucket(&self.bucket)
                .key(path)
                .presigned(PresigningConfig::expires_in(expires_in).map_err(|e| {
                    StorageError::Error(format!("Failed to set presigned config: {}", e))
                })?)
                .await
                .map_err(|e| {
                    StorageError::Error(format!("Failed to generate presigned url: {}", e))
                })?;

            Ok(uri.uri().to_string())
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
            // check if path = "/"
            let objects = if path == "/" || path.is_empty() {
                self.client
                    .list_objects_v2()
                    .bucket(&self.bucket)
                    .send()
                    .await
                    .map_err(|e| StorageError::Error(format!("Failed to list objects: {}", e)))?
            } else {
                self.client
                    .list_objects_v2()
                    .bucket(&self.bucket)
                    .prefix(path)
                    .send()
                    .await
                    .map_err(|e| StorageError::Error(format!("Failed to list objects: {}", e)))?
            };

            Ok(objects
                .contents
                .unwrap_or_else(Vec::new)
                .iter()
                .filter_map(|o| o.key.clone())
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
                .unwrap_or_else(Vec::new)
                .iter()
                .map(|o| {
                    let object_type = match o.storage_class.clone() {
                        Some(storage_class) => storage_class.to_string(),
                        None => "".to_string(),
                    };
                    let key = o.key.as_ref().unwrap_or(&String::new()).clone();
                    let file = Path::new(&key);

                    let size = o.size.unwrap_or_default();

                    let created = match o.last_modified {
                        Some(last_modified) => last_modified.to_string(),
                        None => "".to_string(),
                    };

                    FileInfo {
                        name: file.file_name().unwrap().to_str().unwrap().to_string(),
                        size,
                        object_type,
                        created,
                        suffix: file.extension().unwrap().to_str().unwrap().to_string(),
                    }
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
                .copy_object()
                .copy_source(format!("{}/{}", self.bucket, src))
                .bucket(&self.bucket)
                .key(dest)
                .send()
                .await
                .map_err(|e| StorageError::Error(format!("Failed to copy object: {}", e)))?;

            Ok(true)
        }

        /// Copy objects from the storage bucket
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
            self.client
                .delete_object()
                .bucket(&self.bucket)
                .key(path)
                .send()
                .await
                .map_err(|e| StorageError::Error(format!("Failed to delete object: {}", e)))?;

            Ok(true)
        }

        /// Delete objects from the storage bucket
        ///
        /// # Arguments
        ///
        /// * `path` - Bucket and prefix path to the objects to delete
        ///
        async fn delete_objects(&self, path: &str) -> Result<bool, StorageError> {
            let objects = self.find(path).await?;

            let mut delete_object_ids: Vec<aws_sdk_s3::types::ObjectIdentifier> = vec![];
            for obj in objects {
                let obj_id = aws_sdk_s3::types::ObjectIdentifier::builder()
                    .key(obj)
                    .build()
                    .map_err(|err| {
                        StorageError::Error(format!("Failed to build object identifier: {}", err))
                    })?;
                delete_object_ids.push(obj_id);
            }

            self.client
                .delete_objects()
                .bucket(&self.bucket)
                .delete(
                    aws_sdk_s3::types::Delete::builder()
                        .set_objects(Some(delete_object_ids))
                        .build()
                        .map_err(|err| {
                            StorageError::Error(format!(
                                "Failed to build delete object request: {}",
                                err
                            ))
                        })?,
                )
                .send()
                .await
                .map_err(|e| StorageError::Error(format!("Failed to delete objects: {}", e)))
                .map_err(|e: StorageError| {
                    StorageError::Error(format!("Failed to delete objects: {}", e))
                })?;

            Ok(true)
        }

        // put object stream
    }

    impl AWSStorageClient {
        /// Get an object stream from the storage bucket
        ///
        /// # Arguments
        ///
        /// * `rpath` - The path to the object in the bucket
        ///
        /// # Returns
        ///
        /// A Result with the object stream if successful
        ///
        pub async fn get_object_stream(
            &self,
            rpath: &str,
        ) -> Result<GetObjectOutput, StorageError> {
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

        pub async fn create_multipart_upload(&self, path: &str) -> Result<String, StorageError> {
            let response = self
                .client
                .create_multipart_upload()
                .bucket(&self.bucket)
                .key(path)
                .send()
                .await
                .map_err(|e| {
                    StorageError::Error(format!("Failed to create multipart upload: {}", e))
                })?;

            Ok(response.upload_id.unwrap())
        }

        pub async fn create_multipart_uploader(
            &self,
            path: &str,
            session_url: Option<String>,
        ) -> Result<AWSMulitPartUpload, StorageError> {
            let upload_id = match session_url {
                Some(session_url) => session_url,
                None => self.create_multipart_upload(path).await?,
            };
            AWSMulitPartUpload::new(&self.bucket, path, &upload_id).await
        }

        async fn upload_file_in_chunks(
            &self,
            lpath: &Path,
            uploader: &mut AWSMulitPartUpload,
        ) -> Result<(), StorageError> {
            let file = File::open(lpath)
                .map_err(|e| StorageError::Error(format!("Failed to open file: {}", e)))?;

            // get file size
            let metadata = file
                .metadata()
                .map_err(|e| StorageError::Error(format!("Failed to get file metadata: {}", e)))?;

            let file_size = metadata.len();
            let chunk_size = std::cmp::min(file_size, 1024 * 1024 * 5);

            // calculate the number of parts
            let mut chunk_count = (file_size / chunk_size) + 1;
            let mut size_of_last_chunk = file_size % chunk_size;

            if chunk_count > MAX_CHUNKS {
                return Err(StorageError::Error(
                    "File size is too large for multipart upload".to_string(),
                ));
            }

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

                let stream = uploader
                    .get_next_chunk(lpath, chunk_size, chunk_index, this_chunk)
                    .await?;

                let part_number = (chunk_index as i32) + 1;
                uploader.upload_part(part_number, stream).await?;
            }

            uploader.complete_upload().await?;

            Ok(())
        }

        /// Generate a presigned url for a part in the multipart upload
        /// This needs to be a non-self method because it is called from both client or server
        pub async fn generate_presigned_url_for_part(
            &self,
            part_number: i32,
            path: &str,
            upload_id: &str,
        ) -> Result<String, StorageError> {
            let expires_in = Duration::from_secs(600); // Set expiration time for presigned URL

            let presigned_request = self
                .client
                .upload_part()
                .bucket(&self.bucket)
                .key(path)
                .upload_id(upload_id)
                .part_number(part_number)
                .presigned(PresigningConfig::expires_in(expires_in).map_err(|e| {
                    StorageError::Error(format!("Failed to set presigned config: {}", e))
                })?)
                .await
                .map_err(|e| {
                    StorageError::Error(format!("Failed to generate presigned url: {}", e))
                })?;

            Ok(presigned_request.uri().to_string())
        }
    }

    // For both python and rust, we need to define 2 structs: one for rust that supports async and one for python that does not
    pub struct S3FStorageClient {
        client: AWSStorageClient,
    }

    #[async_trait]
    impl FileSystem<AWSStorageClient> for S3FStorageClient {
        fn name(&self) -> &str {
            "S3FStorageClient"
        }
        fn client(&self) -> &AWSStorageClient {
            &self.client
        }
        async fn new(settings: &OpsmlStorageSettings) -> Self {
            let client = AWSStorageClient::new(settings).await.unwrap();
            Self { client }
        }
    }

    impl S3FStorageClient {
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
                        .create_multipart_uploader(remote_path.to_str().unwrap(), None)
                        .await?;

                    self.client()
                        .upload_file_in_chunks(&stripped_file_path, &mut uploader)
                        .await?;
                }

                Ok(())
            } else {
                let mut uploader = self
                    .client()
                    .create_multipart_uploader(stripped_rpath.to_str().unwrap(), None)
                    .await?;

                self.client()
                    .upload_file_in_chunks(&stripped_lpath, &mut uploader)
                    .await?;
                Ok(())
            }
        }

        pub async fn generate_presigned_url_for_part(
            &self,
            part_number: i32,
            path: &str,
            upload_id: &str,
        ) -> Result<String, StorageError> {
            self.client()
                .generate_presigned_url_for_part(part_number, path, upload_id)
                .await
        }
    }

    #[pyclass]
    pub struct PyS3FSStorageClient {
        client: AWSStorageClient,
        runtime: tokio::runtime::Runtime,
    }

    #[pymethods]
    impl PyS3FSStorageClient {
        #[new]
        fn new(settings: &OpsmlStorageSettings) -> Self {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let client = rt
                .block_on(async { AWSStorageClient::new(settings).await })
                .unwrap();

            Self {
                client,
                runtime: rt,
            }
        }

        #[pyo3(signature = (path=PathBuf::new()))]
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
                            .create_multipart_uploader(remote_path.to_str().unwrap(), None)
                            .await?;

                        self.client
                            .upload_file_in_chunks(&stripped_file_path, &mut uploader)
                            .await?;
                    }

                    Ok(())
                } else {
                    let mut uploader = self
                        .client
                        .create_multipart_uploader(stripped_rpath.to_str().unwrap(), None)
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
        use opsml_settings::config::OpsmlConfig;
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};

        const CHUNK_SIZE: u64 = 1024 * 256;

        fn get_settings() -> OpsmlStorageSettings {
            let bucket = std::env::var("CLOUD_BUCKET_NAME")
                .unwrap_or_else(|_| "opsml-integration".to_string());

            let config = OpsmlConfig::new();
            let mut settings = config.storage_settings();
            settings.storage_uri = bucket;

            settings
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
        async fn test_aws_creds_new() {
            let creds = AWSCreds::new().await;
            assert!(creds.is_ok());
        }

        #[test]
        fn test_aws_storage_client_new() {
            let settings = get_settings();
            let _client = AWSStorageClient::new(&settings);
        }

        #[tokio::test]
        async fn test_aws_storage_client_get_object() {
            let settings = get_settings();
            let client = AWSStorageClient::new(&settings).await.unwrap();

            // should fail since there are no suffixes
            let result = client.get_object("local_path", "remote_path").await;
            assert!(result.is_err()); // Assuming the object does not exist
        }

        #[tokio::test]
        async fn test_s3f_storage_client_put() {
            let settings = get_settings();
            let client = S3FStorageClient::new(&settings).await;

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
        async fn test_aws_storage_client_generate_presigned_url() {
            let settings = get_settings();
            let client = S3FStorageClient::new(&settings).await;

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
        async fn test_aws_large_file_upload() {
            let settings = get_settings();
            let client = S3FStorageClient::new(&settings).await;

            // create file
            let chunk_size = 1024 * 1024 * 5; // 5MB
            let key = create_single_file(&chunk_size);
            let path = Path::new(&key);

            // put the file
            client.put(path, path, false).await.unwrap();

            // cleanup
            client.rm(path, false).await.unwrap();
            std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
        }
    }
}

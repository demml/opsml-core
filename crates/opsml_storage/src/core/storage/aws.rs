#[cfg(feature = "aws_storage")]
pub mod aws_storage {
    use crate::core::storage::base::{get_files, FileInfo, FileSystem, PathExt};
    use crate::core::utils::error::StorageError;
    use aws_config::BehaviorVersion;
    use aws_config::SdkConfig;
    use aws_sdk_s3::operation::get_object::GetObjectOutput;
    use aws_sdk_s3::presigning::PresigningConfig;
    use aws_sdk_s3::primitives::ByteStream;
    use aws_sdk_s3::primitives::Length;
    use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
    use aws_sdk_s3::Client;
    use pyo3::prelude::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;
    use std::str;
    use tokio::runtime::Runtime;

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
        runtime: tokio::runtime::Runtime,
    }

    impl AWSMulitPartUpload {
        pub fn new(bucket: String, path: String, upload_id: String) -> Result<Self, StorageError> {
            // create a resuable runtime for the multipart upload
            let rt = Runtime::new().unwrap();

            let creds = rt
                .block_on(async {
                    let creds = AWSCreds::new().await?;
                    Ok(creds)
                })
                .map_err(|e: StorageError| {
                    StorageError::Error(format!("Failed to create AWS creds: {}", e))
                })?;

            let client = Client::new(&creds.config);

            Ok(Self {
                client,
                bucket,
                path,
                upload_id,
                upload_parts: Vec::new(),
                runtime: rt,
            })
        }

        pub fn upload_part(
            &mut self,
            part_number: i32,
            body: ByteStream,
        ) -> Result<bool, StorageError> {
            let response = self
                .runtime
                .block_on(async {
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
                        .map_err(|e| {
                            StorageError::Error(format!("Failed to upload part: {}", e))
                        })?;

                    Ok(response)
                })
                .map_err(|e: StorageError| {
                    StorageError::Error(format!("Failed to upload part: {}", e))
                })?;

            self.upload_parts.push(
                CompletedPart::builder()
                    .e_tag(response.e_tag.unwrap_or_default())
                    .part_number(part_number)
                    .build(),
            );

            Ok(true)
        }

        pub fn complete(&self) -> Result<(), StorageError> {
            let completed_multipart_upload: CompletedMultipartUpload =
                CompletedMultipartUpload::builder()
                    .set_parts(Some(self.upload_parts.clone()))
                    .build();

            self.runtime
                .block_on(async {
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
                            StorageError::Error(format!(
                                "Failed to complete multipart upload: {}",
                                e
                            ))
                        })?;

                    Ok(())
                })
                .map_err(|e: StorageError| {
                    StorageError::Error(format!("Failed to complete multipart upload: {}", e))
                })
        }

        pub fn get_next_chunk(
            &self,
            path: &Path,
            chunk_size: u64,
            chunk_index: u64,
            this_chunk_size: u64,
        ) -> Result<ByteStream, StorageError> {
            self.runtime.block_on(async {
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
    }

    pub struct AWSStorageClient {
        pub client: Client,
        pub bucket: String,
        runtime: tokio::runtime::Runtime,
    }

    impl AWSStorageClient {
        pub fn new(bucket: String) -> Result<Self, StorageError> {
            // create a resuable runtime for client
            let rt = Runtime::new().unwrap();

            let creds = rt
                .block_on(async {
                    let creds = AWSCreds::new().await?;
                    Ok(creds)
                })
                .map_err(|e: StorageError| {
                    StorageError::Error(format!("Failed to create AWS creds: {}", e))
                })?;

            let client = Client::new(&creds.config);
            Ok(Self {
                client,
                bucket,
                runtime: rt,
            })
        }

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
        pub fn get_object_stream(&self, rpath: &str) -> Result<GetObjectOutput, StorageError> {
            self.runtime.block_on(async {
                let response = self
                    .client
                    .get_object()
                    .bucket(&self.bucket)
                    .key(rpath)
                    .send()
                    .await
                    .map_err(|e| {
                        StorageError::Error(format!("Failed to get object stream: {}", e))
                    })?;
                Ok(response)
            })
        }

        pub fn get_object(&self, lpath: &str, rpath: &str) -> Result<(), StorageError> {
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
            let mut response = self.get_object_stream(rpath.to_str().unwrap())?;

            // write stream to file
            self.runtime.block_on(async {
                // iterate over the stream and write to the file
                while let Some(v) = response.body.next().await {
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
        pub fn generate_presigned_url(
            &self,
            path: &str,
            expiration: u64,
        ) -> Result<String, StorageError> {
            let expires_in = std::time::Duration::from_secs(expiration);

            self.runtime.block_on(async {
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
            })
        }

        pub fn create_multipart_upload(&self, path: &str) -> Result<String, StorageError> {
            self.runtime.block_on(async {
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
            })
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
        pub fn find(&self, path: &str) -> Result<Vec<String>, StorageError> {
            // check if path = "/"
            let objects = if path == "/" || path.is_empty() {
                self.runtime.block_on(async {
                    let response = self
                        .client
                        .list_objects_v2()
                        .bucket(&self.bucket)
                        .send()
                        .await
                        .map_err(|e| {
                            StorageError::Error(format!("Failed to list objects: {}", e))
                        })?;

                    Ok(response)
                })?
            } else {
                self.runtime.block_on(async {
                    let response = self
                        .client
                        .list_objects_v2()
                        .bucket(&self.bucket)
                        .prefix(path)
                        .send()
                        .await
                        .map_err(|e| {
                            StorageError::Error(format!("Failed to list objects: {}", e))
                        })?;

                    Ok(response)
                })?
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
        pub fn find_info(&self, path: &str) -> Result<Vec<FileInfo>, StorageError> {
            let objects = self.runtime.block_on(async {
                let response = self
                    .client
                    .list_objects_v2()
                    .bucket(&self.bucket)
                    .prefix(path)
                    .send()
                    .await
                    .map_err(|e| StorageError::Error(format!("Failed to list objects: {}", e)))?;

                Ok(response)
            })?;

            Ok(objects
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
        pub fn copy_object(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
            self.runtime.block_on(async {
                self.client
                    .copy_object()
                    .copy_source(format!("{}/{}", self.bucket, src))
                    .bucket(&self.bucket)
                    .key(dest)
                    .send()
                    .await
                    .map_err(|e| StorageError::Error(format!("Failed to copy object: {}", e)))?;

                Ok(true)
            })
        }

        /// Copy objects from the storage bucket
        pub fn copy_objects(&self, src: &str, dest: &str) -> Result<bool, StorageError> {
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
        pub fn delete_object(&self, path: &str) -> Result<bool, StorageError> {
            self.runtime.block_on(async {
                self.client
                    .delete_object()
                    .bucket(&self.bucket)
                    .key(path)
                    .send()
                    .await
                    .map_err(|e| StorageError::Error(format!("Failed to delete object: {}", e)))?;

                Ok(true)
            })
        }

        /// Delete objects from the storage bucket
        ///
        /// # Arguments
        ///
        /// * `path` - Bucket and prefix path to the objects to delete
        ///
        pub fn delete_objects(&self, path: &str) -> Result<bool, StorageError> {
            let objects = self.find(path)?;

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

            self.runtime
                .block_on(async {
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
                        .map_err(|e| {
                            StorageError::Error(format!("Failed to delete objects: {}", e))
                        })
                })
                .map_err(|e: StorageError| {
                    StorageError::Error(format!("Failed to delete objects: {}", e))
                })?;

            Ok(true)
        }

        pub fn upload_file_in_chunks(
            &self,
            lpath: &Path,
            rpath: &Path,
            chunk_size: Option<u64>,
        ) -> Result<(), StorageError> {
            let chunk_size = chunk_size.unwrap_or(5 * 1024 * 1024); // 5MB

            let file = File::open(lpath)
                .map_err(|e| StorageError::Error(format!("Failed to open file: {}", e)))?;

            // get file size
            let metadata = file
                .metadata()
                .map_err(|e| StorageError::Error(format!("Failed to get file metadata: {}", e)))?;

            let file_size = metadata.len();

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

            let upload_id = self.create_multipart_upload(rpath.to_str().unwrap())?;

            let mut uploader = AWSMulitPartUpload::new(
                self.bucket.clone(),
                rpath.to_str().unwrap().to_string(),
                upload_id,
            )?;

            for chunk_index in 0..chunk_count {
                let this_chunk = if chunk_count - 1 == chunk_index {
                    size_of_last_chunk
                } else {
                    chunk_size
                };

                let stream = uploader.get_next_chunk(lpath, chunk_size, chunk_index, this_chunk)?;
                let part_number = (chunk_index as i32) + 1;
                uploader.upload_part(part_number, stream)?;
            }

            uploader.complete()?;

            Ok(())
        }
    }

    // For both python and rust, we need to define 2 structs: one for rust that supports async and one for python that does not
    pub struct S3FStorageClient {
        client: AWSStorageClient,
    }

    impl FileSystem for S3FStorageClient {
        fn new(bucket: String) -> Self {
            let client = AWSStorageClient::new(bucket).unwrap();
            Self { client }
        }

        fn find_info(&self, path: &Path) -> Result<Vec<FileInfo>, StorageError> {
            self.client.find_info(path.to_str().unwrap())
        }

        fn find(&self, path: &Path) -> Result<Vec<String>, StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);
            self.client.find(stripped_path.to_str().unwrap())
        }

        fn get(&self, lpath: &Path, rpath: &Path, recursive: bool) -> Result<(), StorageError> {
            // strip the paths
            let stripped_rpath = rpath.strip_path(&self.client.bucket);
            let stripped_lpath = lpath.strip_path(&self.client.bucket);

            if recursive {
                let stripped_lpath_clone = stripped_lpath.clone();

                // list all objects in the path
                let objects = self.client.find(stripped_rpath.to_str().unwrap())?;

                // iterate over each object and get it
                for obj in objects {
                    let file_path = Path::new(obj.as_str());
                    let stripped_path = file_path.strip_path(&self.client.bucket);
                    let relative_path = file_path.relative_path(&stripped_rpath)?;
                    let local_path = stripped_lpath_clone.join(relative_path);

                    self.client.get_object(
                        local_path.to_str().unwrap(),
                        stripped_path.to_str().unwrap(),
                    )?;
                }
            } else {
                self.client.get_object(
                    stripped_lpath.to_str().unwrap(),
                    stripped_rpath.to_str().unwrap(),
                )?;
            }

            Ok(())
        }

        fn put(&self, lpath: &Path, rpath: &Path, recursive: bool) -> Result<(), StorageError> {
            let stripped_lpath = lpath.strip_path(&self.client.bucket);
            let stripped_rpath = rpath.strip_path(&self.client.bucket);

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

                    self.client
                        .upload_file_in_chunks(&stripped_file_path, &remote_path, None)?;
                }

                Ok(())
            } else {
                self.client
                    .upload_file_in_chunks(&stripped_lpath, &stripped_rpath, None)?;
                Ok(())
            }
        }

        fn copy(&self, src: &Path, dest: &Path, recursive: bool) -> Result<(), StorageError> {
            let stripped_src = src.strip_path(&self.client.bucket);
            let stripped_dest = dest.strip_path(&self.client.bucket);

            if recursive {
                self.client.copy_objects(
                    stripped_src.to_str().unwrap(),
                    stripped_dest.to_str().unwrap(),
                )?;
            } else {
                self.client.copy_object(
                    stripped_src.to_str().unwrap(),
                    stripped_dest.to_str().unwrap(),
                )?;
            }

            Ok(())
        }

        fn rm(&self, path: &Path, recursive: bool) -> Result<(), StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);

            if recursive {
                self.client
                    .delete_objects(stripped_path.to_str().unwrap())?;
            } else {
                self.client.delete_object(stripped_path.to_str().unwrap())?;
            }

            Ok(())
        }

        fn exists(&self, path: &Path) -> Result<bool, StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);
            let objects = self.client.find(stripped_path.to_str().unwrap())?;

            Ok(!objects.is_empty())
        }

        fn generate_presigned_url(
            &self,
            path: &Path,
            expiration: u64,
        ) -> Result<String, StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);
            self.client
                .generate_presigned_url(stripped_path.to_str().unwrap(), expiration)
        }
    }

    #[pyclass]
    pub struct PyS3FSStorageClient {
        client: AWSStorageClient,
    }

    #[pymethods]
    impl PyS3FSStorageClient {
        #[new]
        fn new(bucket: String) -> Self {
            let client = AWSStorageClient::new(bucket).unwrap();
            Self { client }
        }

        fn find_info(&self, path: PathBuf) -> Result<Vec<FileInfo>, StorageError> {
            self.client.find_info(path.to_str().unwrap())
        }

        #[pyo3(signature = (path=PathBuf::new()))]
        fn find(&self, path: PathBuf) -> Result<Vec<String>, StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);
            self.client.find(stripped_path.to_str().unwrap())
        }

        #[pyo3(signature = (lpath, rpath, recursive = false))]
        fn get(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError> {
            // strip the paths
            let stripped_rpath = rpath.strip_path(&self.client.bucket);
            let stripped_lpath = lpath.strip_path(&self.client.bucket);

            if recursive {
                let stripped_lpath_clone = stripped_lpath.clone();

                // list all objects in the path
                let objects = self.client.find(stripped_rpath.to_str().unwrap())?;

                // iterate over each object and get it
                for obj in objects {
                    let file_path = Path::new(obj.as_str());
                    let stripped_path = file_path.strip_path(&self.client.bucket);
                    let relative_path = file_path.relative_path(&stripped_rpath)?;
                    let local_path = stripped_lpath_clone.join(relative_path);

                    self.client.get_object(
                        local_path.to_str().unwrap(),
                        stripped_path.to_str().unwrap(),
                    )?;
                }
            } else {
                self.client.get_object(
                    stripped_lpath.to_str().unwrap(),
                    stripped_rpath.to_str().unwrap(),
                )?;
            }

            Ok(())
        }

        #[pyo3(signature = (lpath, rpath, recursive = false))]
        fn put(&self, lpath: PathBuf, rpath: PathBuf, recursive: bool) -> Result<(), StorageError> {
            let stripped_lpath = lpath.strip_path(&self.client.bucket);
            let stripped_rpath = rpath.strip_path(&self.client.bucket);

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

                    self.client
                        .upload_file_in_chunks(&stripped_file_path, &remote_path, None)?;
                }

                Ok(())
            } else {
                self.client
                    .upload_file_in_chunks(&stripped_lpath, &stripped_rpath, None)?;
                Ok(())
            }
        }

        #[pyo3(signature = (src, dest, recursive = false))]
        fn copy(&self, src: PathBuf, dest: PathBuf, recursive: bool) -> Result<(), StorageError> {
            let stripped_src = src.strip_path(&self.client.bucket);
            let stripped_dest = dest.strip_path(&self.client.bucket);

            if recursive {
                self.client.copy_objects(
                    stripped_src.to_str().unwrap(),
                    stripped_dest.to_str().unwrap(),
                )?;
            } else {
                self.client.copy_object(
                    stripped_src.to_str().unwrap(),
                    stripped_dest.to_str().unwrap(),
                )?;
            }

            Ok(())
        }

        #[pyo3(signature = (path, recursive = false))]
        fn rm(&self, path: PathBuf, recursive: bool) -> Result<(), StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);

            if recursive {
                self.client
                    .delete_objects(stripped_path.to_str().unwrap())?;
            } else {
                self.client.delete_object(stripped_path.to_str().unwrap())?;
            }

            Ok(())
        }

        fn exists(&self, path: PathBuf) -> Result<bool, StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);
            let objects = self.client.find(stripped_path.to_str().unwrap())?;

            Ok(!objects.is_empty())
        }

        #[pyo3(signature = (path, expiration = 600))]
        fn generate_presigned_url(
            &self,
            path: PathBuf,
            expiration: u64,
        ) -> Result<String, StorageError> {
            let stripped_path = path.strip_path(&self.client.bucket);
            self.client
                .generate_presigned_url(stripped_path.to_str().unwrap(), expiration)
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};

        const CHUNK_SIZE: u64 = 1024 * 256;

        pub fn get_bucket() -> String {
            std::env::var("CLOUD_BUCKET_NAME").unwrap_or_else(|_| "opsml-integration".to_string())
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
            let bucket = get_bucket();
            let client = AWSStorageClient::new(bucket);
            assert!(client.is_ok());
        }

        #[test]
        fn test_aws_storage_client_get_object() {
            let bucket = get_bucket();
            let client = AWSStorageClient::new(bucket).unwrap();

            // should fail since there are no suffixes
            let result = client.get_object("local_path", "remote_path");
            assert!(result.is_err()); // Assuming the object does not exist
        }

        #[test]
        fn test_s3f_storage_client_put() {
            let bucket = get_bucket();
            let client = S3FStorageClient::new(bucket);

            //
            let dirname = create_nested_data(&CHUNK_SIZE);

            let lpath = Path::new(&dirname);
            let rpath = Path::new(&dirname);

            // put the file
            client.put(lpath, rpath, true).unwrap();

            // check if the file exists
            let exists = client.exists(rpath).unwrap();
            assert!(exists);

            // list all files
            let files = client.find(rpath).unwrap();
            assert_eq!(files.len(), 2);

            // list files with info
            let files = client.find_info(rpath).unwrap();
            assert_eq!(files.len(), 2);

            // download the files
            let new_path = uuid::Uuid::new_v4().to_string();
            let new_path = Path::new(&new_path);

            client.get(new_path, rpath, true).unwrap();

            // check if the files are the same
            let files = get_files(rpath).unwrap();
            let new_files = get_files(new_path).unwrap();

            assert_eq!(files.len(), new_files.len());

            // copy the files
            // create a new path
            let copy_path = uuid::Uuid::new_v4().to_string();
            let copy_path = Path::new(&copy_path);
            client.copy(rpath, copy_path, true).unwrap();
            let files = client.find(copy_path).unwrap();
            assert_eq!(files.len(), 2);

            // cleanup
            std::fs::remove_dir_all(&dirname).unwrap();
            std::fs::remove_dir_all(new_path).unwrap();
            client.rm(rpath, true).unwrap();
            client.rm(copy_path, true).unwrap();

            // check if the file exists
            let exists = client.exists(rpath).unwrap();
            assert!(!exists);
        }

        #[test]
        fn test_aws_storage_client_generate_presigned_url() {
            let bucket = get_bucket();
            let client = S3FStorageClient::new(bucket);

            // create file
            let key = create_single_file(&CHUNK_SIZE);
            let path = Path::new(&key);

            // put the file
            client.put(path, path, false).unwrap();

            // generate presigned url
            let url = client.generate_presigned_url(path, 3600).unwrap();
            assert!(!url.is_empty());

            // cleanup
            client.rm(path, false).unwrap();
            std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
        }

        #[test]
        fn test_aws_large_file_upload() {
            let bucket = get_bucket();
            let client = S3FStorageClient::new(bucket);

            // create file
            let chunk_size = 1024 * 1024 * 5; // 5MB
            let key = create_single_file(&chunk_size);
            let path = Path::new(&key);

            // put the file
            client.put(path, path, false).unwrap();

            // cleanup
            client.rm(path, false).unwrap();
            std::fs::remove_dir_all(path.parent().unwrap()).unwrap();
        }
    }
}
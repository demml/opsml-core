use crate::core::utils::error::StorageError;
use aws_config::BehaviorVersion;
use aws_config::SdkConfig;
use aws_sdk_s3::operation::get_object::GetObjectOutput;
use aws_sdk_s3::presigning::PresigningConfig;
use aws_sdk_s3::primitives::ByteStream;
use aws_sdk_s3::primitives::Length;
use aws_sdk_s3::types::{CompletedMultipartUpload, CompletedPart};
use aws_sdk_s3::Client;
use bytes::Bytes;
use pyo3::prelude::*;
use pyo3::pyclass;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

const MAX_CHUNKS: u64 = 10000;

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
    pub async fn new(
        bucket: String,
        path: String,
        upload_id: String,
    ) -> Result<Self, StorageError> {
        let creds = AWSCreds::new().await?;
        let client = Client::new(&creds.config);

        Ok(Self {
            client,
            bucket,
            path,
            upload_id,
            upload_parts: Vec::new(),
        })
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

    pub async fn complete(&self) -> Result<(), StorageError> {
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

    pub async fn get_object(&self, lpath: &str, rpath: &str) -> Result<(), StorageError> {
        let mut response = self.get_object_stream(rpath).await?;

        // create and open lpath file
        let prefix = Path::new(lpath).parent().unwrap();

        if !prefix.exists() {
            // create the directory if it does not exist and skip errors
            std::fs::create_dir_all(prefix)
                .map_err(|e| StorageError::Error(format!("Unable to create directory: {}", e)))?;
        }

        // create and open lpath file
        let mut file = File::create(lpath)
            .map_err(|e| StorageError::Error(format!("Unable to create file: {}", e)))?;

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
    pub async fn generate_presigned_url(
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
            .map_err(|e| StorageError::Error(format!("Failed to generate presigned url: {}", e)))?;

        Ok(uri.uri().to_string())
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

    /// Delete an object from the storage bucket
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the object in the bucket
    ///
    pub async fn delete_object(&self, path: &str) -> Result<bool, StorageError> {
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
    pub async fn delete_objects(&self, path: &str) -> Result<bool, StorageError> {
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
            .map_err(|e| StorageError::Error(format!("Failed to delete objects: {}", e)))?;

        Ok(true)
    }

    pub async fn upload_file_in_chunks(
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

        // if the last chunk is empty, reduce the number of parts
        if size_of_last_chunk == 0 {
            size_of_last_chunk = chunk_size;
            chunk_count -= 1;
        }

        let upload_id = self
            .create_multipart_upload(rpath.to_str().unwrap())
            .await?;

        let mut uploader = AWSMulitPartUpload::new(
            self.bucket.clone(),
            rpath.to_str().unwrap().to_string(),
            upload_id,
        )
        .await?;

        for chunk_index in 0..chunk_count {
            let this_chunk = if chunk_count - 1 == chunk_index {
                size_of_last_chunk
            } else {
                chunk_size
            };

            let stream = ByteStream::read_from()
                .path(lpath)
                .offset(chunk_index * chunk_size)
                .length(Length::Exact(this_chunk))
                .build()
                .await
                .unwrap();

            let part_number = (chunk_index as i32) + 1;

            uploader.upload_part(part_number, stream).await?;
        }

        uploader.complete().await?;

        Ok(())
    }
}

// For both python and rust, we need to define 2 structs: one for rust that supports async and one for python that does not

pub struct S3FStorageClient {
    client: AWSStorageClient,
}

impl S3FStorageClient {
    pub async fn new(bucket: String) -> Self {
        let client = AWSStorageClient::new(bucket).await.unwrap();
        S3FStorageClient { client }
    }

    pub async fn find(&self, path: PathBuf) -> Vec<String> {
        let stripped_path = path
            .strip_prefix(&self.client.bucket)
            .unwrap_or(&path)
            .to_str()
            .unwrap();

        self.client.find(stripped_path).await.unwrap()
    }
}

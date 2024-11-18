use crate::core::storage::local::LocalStorageClient;
use crate::core::utils::error::{ApiError, StorageError};

use crate::core::storage::base::{FileInfo, StorageClient, StorageSettings};
use async_trait::async_trait;
use aws_smithy_types::byte_stream::ByteStream;
use aws_smithy_types::byte_stream::Length;
use futures::future::join_all;
use futures::stream::TryStreamExt;
use futures::TryFutureExt;
use google_cloud_storage::http::objects::upload;
use reqwest::multipart::{Form, Part};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Body, Client,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::task::JoinHandle;
use tokio_util::codec::{BytesCodec, FramedRead};

const TIMEOUT_SECS: u64 = 30;

const CHUNK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone)]
pub enum RequestType {
    Get,
    Post,
    Put,
}

#[derive(Debug, Clone)]
pub enum Routes {
    Multipart,
    List,
    ListInfo,
    Files,
    ValidateFile,
}

impl Routes {
    pub fn as_str(&self) -> &str {
        match self {
            Routes::Files => "opsml/files",
            Routes::Multipart => "opsml/files/multipart",
            Routes::ValidateFile => "opsml/files/validate",
            Routes::List => "opsml/files/list",
            Routes::ListInfo => "opsml/files/list_info",
        }
    }
}

pub struct MultiPartUploader {}

impl MultiPartUploader {
    pub async fn get_next_chunk(
        path: &Path,
        chunk_size: u64,
        chunk_index: u64,
        this_chunk_size: u64,
    ) -> Result<ByteStream, ApiError> {
        let stream = ByteStream::read_from()
            .path(path)
            .offset(chunk_index * chunk_size)
            .length(Length::Exact(this_chunk_size))
            .build()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to get next chunk: {}", e)))?;

        Ok(stream)
    }

    pub async fn upload_part(
        client: ApiClient,
        chunk: ByteStream,
        chunk_index: u64,
        chunk_size: u64,
        first_chunk: u64,
        last_chunk: u64,
        rpath: &str,
        total_size: u64,
    ) -> Result<(), ApiError> {
        let data = chunk
            .collect()
            .await
            .map_err(|e| ApiError::Error(format!("Unable to collect chunk data: {}", e)))?
            .into_bytes();

        let filename = format!("chunk_{}", chunk_index);

        let rpath = Path::new(rpath).

        let part = Part::stream_with_length(data, chunk_size)
            .file_name(format!("chunk_{}", chunk_index))
            .mime_str("application/octet-stream")
            .map_err(|e| ApiError::Error(format!("Failed to create mime type: {}", e)))?;

        let form = Form::new().part("file", part);

        let token = client.auth_token.clone().unwrap_or_default();
        let mut headers = HeaderMap::new();

        // add chunk size, chunk index, and upload uri to headers
        headers.insert(
            "Chunk-Size",
            HeaderValue::from_str(&chunk_size.to_string()).map_err(|e| {
                ApiError::Error(format!("Failed to create content length header: {}", e))
            })?,
        );

        headers.insert(
            "Chunk-Range",
            HeaderValue::from_str(&format!("{}/{}", first_chunk, last_chunk)).map_err(|e| {
                ApiError::Error(format!("Failed to create content range header: {}", e))
            })?,
        );

        headers.insert(
            "Part-Index",
            HeaderValue::from_str(&format!("{}", chunk_index)).map_err(|e| {
                ApiError::Error(format!("Failed to create content range header: {}", e))
            })?,
        );

        headers.insert(
            "Upload-Uri",
            HeaderValue::from_str(upload_uri).map_err(|e| {
                ApiError::Error(format!("Failed to create upload uri header: {}", e))
            })?,
        );

        headers.insert(
            "Total-Size",
            HeaderValue::from_str(&total_size.to_string()).map_err(|e| {
                ApiError::Error(format!("Failed to create total size header: {}", e))
            })?,
        );

        let response = client
            .client
            .post(format!(
                "{}/{}",
                client.base_url,
                Routes::Multipart.as_str()
            ))
            .headers(headers)
            .bearer_auth(token)
            .multipart(form)
            .send()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to send request with error: {}", e)))?;

        Ok(())
    }

    pub async fn validate_file(
        mut client: ApiClient,
        upload_uri: &str,
        total_size: u64,
        total_chunks: u64,
    ) -> Result<(), ApiError> {
        let mut body = HashMap::new();

        body.insert("Upload-Uri", upload_uri.to_string());
        body.insert("Total-Size", total_size.to_string());
        body.insert("file_name", "test".to_string());
        body.insert("Total-Chunks", total_chunks.to_string());

        let response = client.request_with_retry(
            Routes::ValidateFile,
            RequestType::Post,
            Some(json!(body)),
            None,
            None,
        );

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    requires_auth: bool,
    auth_token: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl ApiClient {
    pub async fn new(
        base_url: String,
        use_auth: bool,
        path_prefix: String,
        username: Option<String>,
        password: Option<String>,
        token: Option<String>,
    ) -> Result<Self, ApiError> {
        // setup headers
        let mut headers = HeaderMap::new();
        let client_builder =
            Client::builder().timeout(std::time::Duration::from_secs(TIMEOUT_SECS));
        let base_url = format!("{}/{}", base_url, path_prefix);

        if let Some(token) = token {
            headers.insert(
                "X-Prod-Token",
                HeaderValue::from_str(&token).map_err(|e| {
                    ApiError::Error(format!("Failed to create header with error: {}", e))
                })?,
            );
        }
        let client = client_builder
            .default_headers(headers)
            .build()
            .map_err(|e| ApiError::Error(format!("Failed to create client with error: {}", e)))?;

        let mut api_client = Self {
            client,
            base_url,
            requires_auth: use_auth,
            auth_token: None,
            username: username.clone(),
            password: password.clone(),
        };

        if use_auth {
            if username.is_none() || password.is_none() {
                return Err(ApiError::Error(
                    "Username and password must be provided for authenticated requests".to_string(),
                ));
            }
            api_client.refresh_token().await?;
        }

        Ok(api_client)
    }

    async fn refresh_token(&mut self) -> Result<(), ApiError> {
        if !self.requires_auth {
            return Ok(());
        }

        let form = json!({
            "username": self.username,
            "password": self.password
        });

        let response = self
            .client
            .post(format!("{}/{}", self.base_url, "auth/token"))
            .json(&form)
            .send()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to send request with error: {}", e)))?
            .json::<Value>()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to parse response with error: {}", e)))?;

        // check if the response has "access_token" field
        let token = response["access_token"]
            .as_str()
            .ok_or_else(|| ApiError::Error("Failed to get access token".to_string()))?;

        self.auth_token = Some(token.to_string());

        Ok(())
    }

    async fn request(
        self,
        route: Routes,
        request_type: RequestType,
        body_params: Option<Value>,
        query_params: HashMap<String, String>,
        headers: Option<HeaderMap>,
    ) -> Result<Value, ApiError> {
        let headers = headers.unwrap_or(HeaderMap::new());

        let token = self.auth_token.unwrap_or("".to_string());
        let response = match request_type {
            RequestType::Get => self
                .client
                .get(format!("{}/{}", self.base_url, route.as_str()))
                // add bearer token if it exists
                .headers(headers)
                .query(&query_params)
                .bearer_auth(token)
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!("Failed to send request with error: {}", e))
                })?,
            RequestType::Post => self
                .client
                .post(format!("{}/{}", self.base_url, route.as_str()))
                .headers(headers)
                .json(&body_params)
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!("Failed to send request with error: {}", e))
                })?,
            RequestType::Put => self
                .client
                .put(format!("{}/{}", self.base_url, route.as_str()))
                .headers(headers)
                .json(&body_params)
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!("Failed to send request with error: {}", e))
                })?,
        };

        let response = response
            .json::<Value>()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to parse response with error: {}", e)))?;

        Ok(response)
    }

    pub async fn request_with_retry(
        &mut self,
        route: Routes,
        request_type: RequestType,
        body_params: Option<Value>,
        query_params: Option<HashMap<String, String>>,
        headers: Option<HeaderMap>,
    ) -> Result<Value, ApiError> {
        // this will attempt to send a request. If the request fails, it will refresh the token and try again. If it fails all 3 times it will return an error
        let mut attempts = 0;
        let max_attempts = 3;
        let mut response: Result<Value, ApiError>;

        let query_params = query_params.unwrap_or(HashMap::new());

        loop {
            attempts += 1;

            let client = self.clone();
            response = client
                .request(
                    route.clone(),
                    request_type.clone(),
                    body_params.clone(),
                    query_params.clone(),
                    headers.clone(),
                )
                .await;

            if response.is_ok() || attempts >= max_attempts {
                break;
            }

            if response.is_err() {
                self.refresh_token().await.map_err(|e| {
                    ApiError::Error(format!("Failed to refresh token with error: {}", e))
                })?;
            }
        }

        let response = response
            .map_err(|e| ApiError::Error(format!("Failed to send request with error: {}", e)))?;

        Ok(response)
    }

    /// creates a multipart upload request. Returns the upload uri
    ///
    /// # Arguments
    ///
    /// * `rpath` - the path to the file to upload
    ///
    /// # Returns
    ///
    /// * `Result<String, ApiError>` - the upload uri
    async fn create_multipart_upload(&mut self, rpath: &Path) -> Result<String, ApiError> {
        let mut query_params = HashMap::new();
        query_params.insert("path".to_string(), rpath.to_str().unwrap().to_string());

        let result = self
            .request_with_retry(
                Routes::Multipart,
                RequestType::Get,
                None,
                Some(query_params),
                None,
            )
            .await?;

        let upload_uri = result["upload_uri"]
            .as_str()
            .ok_or_else(|| ApiError::Error("Failed to get upload uri".to_string()))?;

        Ok(upload_uri.to_string())
    }

    async fn upload_file_in_chunks(
        &mut self,
        lpath: &Path,
        rpath: &Path,
        chunk_size: Option<u64>,
    ) -> Result<(), ApiError> {
        let cloned_lpath = lpath.to_path_buf();

        let file = File::open(lpath)
            .await
            .map_err(|e| ApiError::Error(format!("Failed to open file: {}", e)))?;

        let metadata = file
            .metadata()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to get file metadata: {}", e)))?;

        let file_size = metadata.len();

        // Set chunk size to 5MB if None, or to file size if greater than file size
        let chunk_size = chunk_size.unwrap_or(1024 * 1024 * 5).min(file_size);

        // Calculate the number of parts
        let chunk_count = (file_size + chunk_size - 1) / chunk_size;
        let size_of_last_chunk = file_size % chunk_size;

        let session_uri = self.create_multipart_upload(rpath).await?;
        let mut futures = Vec::new();

        for chunk_index in 0..chunk_count {
            let this_chunk = if chunk_index == chunk_count - 1 && size_of_last_chunk != 0 {
                size_of_last_chunk
            } else {
                chunk_size
            };

            let first_byte = chunk_index * chunk_size;
            let last_byte = first_byte + this_chunk - 1;
            let client = self.clone();
            let upload_uri = session_uri.clone();
            let cloned_lpath = cloned_lpath.clone();
            let total_size = file_size.clone();

            let future = tokio::spawn(async move {
                let stream = MultiPartUploader::get_next_chunk(
                    Path::new(&cloned_lpath),
                    chunk_size,
                    chunk_index,
                    this_chunk,
                )
                .await
                .map_err(|e| ApiError::Error(format!("Failed to get next chunk: {}", e)))?;

                MultiPartUploader::upload_part(
                    client,
                    stream,
                    chunk_index,
                    chunk_size,
                    first_byte,
                    last_byte,
                    &upload_uri,
                    total_size,
                )
                .await?;
                Ok::<(), ApiError>(())
            });

            futures.push(future);
        }

        join_all(futures).await;

        // need to add logic to combine the parts
        MultiPartUploader::validate_file(client, upload_uri, total_size, total_chunks)

        Ok(())
    }

    pub async fn upload_file(&self, route: &str, file_path: PathBuf) -> Result<Value, ApiError> {
        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| ApiError::Error("Invalid filename".to_string()))?
            .to_string();

        // Open file as async stream
        let file = File::open(file_path)
            .await
            .map_err(|e| ApiError::Error(format!("Failed to open file with error: {}", e)))?;

        // Get file size for Content-Length header
        let metadata = file
            .metadata()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to get file metadata: {}", e)))?;

        // get filename from pathBuf

        // create stream of bytes
        let stream = FramedRead::with_capacity(file, BytesCodec::new(), CHUNK_SIZE);
        let body = Body::wrap_stream(stream);

        let mut headers = header::HeaderMap::new();

        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );

        headers.insert(
            header::CONTENT_LENGTH,
            HeaderValue::from_str(&metadata.len().to_string()).map_err(|e| {
                ApiError::Error(format!("Failed to create content length header: {}", e))
            })?,
        );

        headers.insert(
            "WRITE_PATH",
            HeaderValue::from_str(&file_name).map_err(|e| {
                ApiError::Error(format!("Failed to create write path header: {}", e))
            })?,
        );

        //add auth
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!(
                "Bearer {}",
                self.auth_token.clone().unwrap_or_default()
            ))
            .map_err(|e| ApiError::Error(format!("Failed to create auth header: {}", e)))?,
        );

        let response = self
            .client
            .post(format!("{}/{}", self.base_url, route))
            .headers(headers)
            .body(body)
            .send()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to send request with error: {}", e)))?;

        let response = response
            .json::<Value>()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to parse response with error: {}", e)))?;

        Ok(response)
    }

    pub async fn upload_file_with_retry(
        &mut self,
        route: &str,
        file_path: PathBuf,
    ) -> Result<Value, ApiError> {
        let mut attempts = 0;
        let max_attempts = 3;
        let mut response: Result<Value, ApiError>;

        loop {
            attempts += 1;

            let client = self.clone();
            response = client.upload_file(route, file_path.clone()).await;

            if response.is_ok() || attempts >= max_attempts {
                break;
            }

            if response.is_err() {
                self.refresh_token().await.map_err(|e| {
                    ApiError::Error(format!("Failed to refresh token with error: {}", e))
                })?;
            }
        }

        let response = response
            .map_err(|e| ApiError::Error(format!("Failed to send request with error: {}", e)))?;

        Ok(response)
    }

    async fn download_to_file(
        &self,
        route: Routes,
        local_path: PathBuf,
        read_path: PathBuf,
    ) -> Result<Value, ApiError> {
        // Create parent directories if they don't exist
        if let Some(parent) = local_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ApiError::Error(format!("Failed to create directories: {}", e)))?;
        }

        // Convert read_path to string for header
        let read_path_str = read_path
            .to_str()
            .ok_or_else(|| ApiError::Error("Invalid read path".to_string()))?;

        // Set up headers
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "READ_PATH",
            HeaderValue::from_str(read_path_str).map_err(|e| {
                ApiError::Error(format!("Failed to create read path header: {}", e))
            })?,
        );

        // Make streaming GET request
        let response = self
            .client
            .get(format!("{}/{}", self.base_url, route.as_str()))
            .headers(headers)
            .bearer_auth(self.auth_token.clone().unwrap_or_default())
            .send()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to send request: {}", e)))?;

        // Check if request was successful
        if !response.status().is_success() {
            return Err(ApiError::Error(format!(
                "Download failed with status: {}",
                response.status()
            )));
        }

        // Create file
        let mut file = File::create(&local_path)
            .await
            .map_err(|e| ApiError::Error(format!("Failed to create file: {}", e)))?;

        // Stream chunks to file
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream
            .try_next()
            .await
            .map_err(|e| ApiError::Error(format!("Failed to read chunk: {}", e)))?
        {
            tokio::io::AsyncWriteExt::write_all(&mut file, &chunk)
                .await
                .map_err(|e| ApiError::Error(format!("Failed to write chunk: {}", e)))?;
        }

        Ok(json!({
            "status": "success",
            "path": local_path.to_string_lossy()
        }))
    }

    pub async fn download_to_file_with_retry(
        &mut self,
        route: Routes,
        local_path: PathBuf,
        read_path: PathBuf,
    ) -> Result<Value, ApiError> {
        let mut attempts = 0;
        let max_attempts = 3;
        let mut response: Result<Value, ApiError>;

        loop {
            attempts += 1;

            let client = self.clone();
            response = client
                .download_to_file(route, local_path.clone(), read_path.clone())
                .await;

            if response.is_ok() || attempts >= max_attempts {
                break;
            }

            if response.is_err() {
                self.refresh_token().await.map_err(|e| {
                    ApiError::Error(format!("Failed to refresh token with error: {}", e))
                })?;
            }
        }

        let response = response
            .map_err(|e| ApiError::Error(format!("Failed to send request with error: {}", e)))?;

        Ok(response)
    }
}

pub struct ApiClientArgs {
    pub base_url: String,
    pub use_auth: bool,
    pub path_prefix: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub token: Option<String>,
}

pub struct HttpStorageClient {
    client: ApiClient,
    storage_client: LocalStorageClient,
    pub bucket: PathBuf,
}

impl HttpStorageClient {
    fn get_http_kwargs(kwargs: HashMap<String, String>) -> Result<ApiClientArgs, StorageError> {
        let base_url = kwargs.get("base_url").ok_or(StorageError::Error(
            "base_url not found in kwargs".to_string(),
        ))?;

        let auth_auth = kwargs
            .get("use_auth")
            .ok_or(StorageError::Error("auth not found in kwargs".to_string()))?;

        let path_prefix = kwargs.get("path_prefix").ok_or(StorageError::Error(
            "path_prefix not found in kwargs".to_string(),
        ))?;

        let username = kwargs.get("username");
        let password = kwargs.get("password");
        let token = kwargs.get("token");

        Ok(ApiClientArgs {
            base_url: base_url.to_string(),
            use_auth: auth_auth.parse().unwrap(),
            path_prefix: path_prefix.to_string(),
            username: username.map(|s| s.to_string()),
            password: password.map(|s| s.to_string()),
            token: token.map(|s| s.to_string()),
        })
    }
}

#[async_trait]
impl StorageClient for HttpStorageClient {
    async fn bucket(&self) -> &str {
        self.bucket.to_str().unwrap()
    }

    async fn new(settings: StorageSettings) -> Result<Self, StorageError> {
        let bucket = PathBuf::from(settings.storage_uri.clone());

        // bucket should be a dir. Check if it exists. If not, create it
        if !bucket.exists() {
            std::fs::create_dir_all(&bucket)
                .map_err(|e| {
                    StorageError::Error(format!("Unable to create bucket directory: {}", e))
                })
                .unwrap();
        }

        let api_kwargs = Self::get_http_kwargs(settings.kwargs.clone())?;

        let client = ApiClient::new(
            api_kwargs.base_url,
            api_kwargs.use_auth,
            api_kwargs.path_prefix,
            api_kwargs.username,
            api_kwargs.password,
            api_kwargs.token,
        )
        .await
        .map_err(|e| StorageError::Error(format!("Failed to create api client: {}", e)))?;

        let storage_client = LocalStorageClient::new(settings).await?;

        Ok(Self {
            client,
            bucket,
            storage_client,
        })
    }

    async fn find(&self, path: &str) -> Result<Vec<String>, StorageError> {
        let mut params = HashMap::new();
        params.insert("path".to_string(), path.to_string());

        // need to clone because self is borrowed
        let mut client = self.client.clone();
        let response = client
            .request_with_retry(Routes::List, RequestType::Get, None, Some(params), None)
            .await
            .map_err(|e| StorageError::Error(format!("Failed to get files: {}", e)))?;

        // convert Value to Vec<String>
        let files = response["files"]
            .as_array()
            .ok_or_else(|| StorageError::Error("Failed to get files".to_string()))?
            .iter()
            .map(|f| f.as_str().unwrap().to_string())
            .collect();

        Ok(files)
    }

    async fn find_info(&self, path: &str) -> Result<Vec<FileInfo>, StorageError> {
        let mut params = HashMap::new();
        params.insert("path".to_string(), path.to_string());

        // need to clone because self is borrowed
        let mut client = self.client.clone();
        let response = client
            .request_with_retry(Routes::ListInfo, RequestType::Get, None, Some(params), None)
            .await
            .map_err(|e| StorageError::Error(format!("Failed to get files: {}", e)))?;

        // convert Value to Vec<FileInfo>
        let files = response["files"]
            .as_array()
            .ok_or_else(|| StorageError::Error("Failed to get files".to_string()))?
            .iter()
            .map(|f| {
                let name = f["name"].as_str().unwrap().to_string();
                let size = f["size"].as_i64().unwrap();
                let object_type = f["object_type"].as_str().unwrap().to_string();
                let created = f["created"].as_str().unwrap().to_string();
                let suffix = f["suffix"].as_str().unwrap().to_string();
                FileInfo {
                    name,
                    size,
                    object_type,
                    created,
                    suffix,
                }
            })
            .collect();

        Ok(files)
    }

    async fn get_object(&self, local_path: &str, remote_path: &str) -> Result<(), StorageError> {
        let local_path = PathBuf::from(local_path);
        let remote_path = PathBuf::from(remote_path);

        let response = self
            .client
            .download_to_file_with_retry(Routes::Multipart, local_path, remote_path)
            .await
            .map_err(|e| StorageError::Error(format!("Failed to download file: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Server, ServerGuard};
    use std::fs;
    use tokio;

    const PATH_PREFIX: &str = "opsml";

    async fn setup_server() -> (ServerGuard, String) {
        let server = Server::new_async().await;
        let server_url = server.url();
        (server, server_url)
    }

    #[tokio::test]
    async fn test_api_client_no_auth() {
        let (mut server, server_url) = setup_server().await;

        let _mock = server
            .mock("GET", "/opsml/files")
            .with_status(200)
            .with_body(r#"{"status": "ok"}"#)
            .create();

        let mut api_client =
            ApiClient::new(server_url, false, PATH_PREFIX.to_string(), None, None, None)
                .await
                .unwrap();

        let response = api_client
            .request_with_retry(Routes::Files, RequestType::Get, None, None, None)
            .await
            .unwrap();

        assert_eq!(response["status"], "ok");
    }

    #[tokio::test]
    async fn test_api_client_with_auth() {
        let (mut server, server_url) = setup_server().await;

        // Mock auth token endpoint
        let _token_mock = server
            .mock("POST", "/opsml/auth/token")
            .with_status(200)
            .with_body(r#"{"access_token": "test_token"}"#)
            .expect(1)
            .create();

        // Mock protected endpoint
        let _protected_mock = server
            .mock("GET", "/opsml/files")
            .match_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_body(r#"{"status": "authenticated"}"#)
            .expect(1)
            .create();

        let mut api_client = ApiClient::new(
            server_url,
            true,
            PATH_PREFIX.to_string(),
            Some("username".to_string()),
            Some("password".to_string()),
            None,
        )
        .await
        .unwrap();

        let response = api_client
            .request_with_retry(Routes::Files, RequestType::Get, None, None, None)
            .await
            .unwrap();

        assert_eq!(response["status"], "authenticated");
    }

    #[tokio::test]
    async fn test_request_with_retry_success() {
        let (mut server, server_url) = setup_server().await;

        // Mock auth token endpoint
        let _token_mock = server
            .mock("POST", "/opsml/auth/token")
            .with_status(200)
            .with_body(r#"{"access_token": "test_token"}"#)
            .expect(1)
            .create();

        // Mock endpoint that succeeds
        let _success_mock = server
            .mock("GET", "/opsml/files")
            .match_header("Authorization", "Bearer test_token")
            .with_status(200)
            .with_body(r#"{"status": "success"}"#)
            .expect(1)
            .create();

        let mut api_client = ApiClient::new(
            server_url,
            true,
            PATH_PREFIX.to_string(),
            Some("username".to_string()),
            Some("password".to_string()),
            None,
        )
        .await
        .unwrap();

        let response = api_client
            .request_with_retry(Routes::Files, RequestType::Get, None, None, None)
            .await
            .unwrap();

        assert_eq!(response["status"], "success");
    }

    #[tokio::test]
    async fn test_request_with_retry_failure() {
        let (mut server, server_url) = setup_server().await;

        // Mock auth token endpoint - will be called multiple times
        let _token_mock = server
            .mock("POST", "/opsml/auth/token")
            .with_status(200)
            .with_body(r#"{"access_token": "test_token"}"#)
            .expect(3)
            .create();

        // Mock endpoint that fails with 401 three times
        let _failure_mock = server
            .mock("GET", "/opsml/files")
            .match_header("Authorization", "Bearer test_token")
            .with_status(401)
            .expect(3)
            .create();

        let mut api_client = ApiClient::new(
            server_url,
            true,
            PATH_PREFIX.to_string(),
            Some("username".to_string()),
            Some("password".to_string()),
            None,
        )
        .await
        .unwrap();

        let result = api_client
            .request_with_retry(Routes::Files, RequestType::Get, None, None, None)
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_request_with_retry_auth_refresh() {
        let (mut server, server_url) = setup_server().await;

        // Mock auth token endpoint - first token
        let _initial_token_mock = server
            .mock("POST", "/opsml/auth/token")
            .with_status(200)
            .with_body(r#"{"access_token": "initial_token"}"#)
            .expect(1)
            .create();

        // Mock auth token endpoint - refresh token
        let _refresh_token_mock = server
            .mock("POST", "/opsml/auth/token")
            .with_status(200)
            .with_body(r#"{"access_token": "refreshed_token"}"#)
            .expect(1)
            .create();

        // Mock protected endpoint - first attempt fails with 401
        let _first_attempt_mock = server
            .mock("GET", "/opsml/test")
            .match_header("Authorization", "Bearer initial_token")
            .with_status(401)
            .expect(1)
            .create();

        // Mock protected endpoint - second attempt succeeds with new token
        let _second_attempt_mock = server
            .mock("GET", "/opsml/files")
            .match_header("Authorization", "Bearer refreshed_token")
            .with_status(200)
            .with_body(r#"{"status": "success_after_refresh"}"#)
            .expect(1)
            .create();

        let mut api_client = ApiClient::new(
            server_url,
            true,
            PATH_PREFIX.to_string(),
            Some("username".to_string()),
            Some("password".to_string()),
            None,
        )
        .await
        .unwrap();

        let response = api_client
            .request_with_retry(Routes::Files, RequestType::Get, None, None, None)
            .await
            .unwrap();

        assert_eq!(response["status"], "success_after_refresh");

        _first_attempt_mock.assert();
        _second_attempt_mock.assert();
        _initial_token_mock.assert();
        _refresh_token_mock.assert();
    }

    #[tokio::test]
    async fn test_upload_file() {
        let (mut server, server_url) = setup_server().await;

        // Create temp file for testing
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_upload.txt");
        fs::write(&file_path, "test content").unwrap();

        // Mock upload endpoint
        let upload_mock = server
            .mock("POST", "/opsml/upload")
            .match_header("Content-Type", "application/octet-stream")
            .match_header("Content-Length", "12") // length of "test content"
            .match_header("WRITE_PATH", "test_upload.txt")
            .with_status(200)
            .with_body(r#"{"status": "uploaded"}"#)
            .expect(1)
            .create();

        let mut api_client =
            ApiClient::new(server_url, false, PATH_PREFIX.to_string(), None, None, None)
                .await
                .unwrap();

        let response = api_client
            .upload_file_with_retry("upload", file_path.clone())
            .await
            .unwrap();

        // Verify response
        assert_eq!(response["status"], "uploaded");

        // Verify mock was called
        upload_mock.assert();

        // Cleanup
        fs::remove_file(file_path).unwrap();
    }

    #[tokio::test]
    async fn test_download_to_file_with_retry_success() {
        let (mut server, server_url) = setup_server().await;

        // Create temp paths
        let temp_dir = std::env::temp_dir();
        let local_path = temp_dir.join("test_download.txt");
        let read_path = PathBuf::from("remote/test_file.txt");

        // Mock successful download endpoint
        let download_mock = server
            .mock("GET", "/opsml/download")
            .match_header("READ_PATH", read_path.to_str().unwrap())
            .with_status(200)
            .with_body("test content")
            .expect(1)
            .create();

        let mut api_client =
            ApiClient::new(server_url, false, PATH_PREFIX.to_string(), None, None, None)
                .await
                .unwrap();

        let response = api_client
            .download_to_file_with_retry("download", local_path.clone(), read_path)
            .await
            .unwrap();

        // Verify response
        assert_eq!(response["status"], "success");
        assert_eq!(response["path"], local_path.to_str().unwrap());

        // Verify file contents
        let content = fs::read_to_string(&local_path).unwrap();
        assert_eq!(content, "test content");

        // Verify mock was called
        download_mock.assert();

        // Cleanup
        fs::remove_file(local_path).unwrap();
    }
}

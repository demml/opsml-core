use crate::core::utils::error::{ApiError, StorageError};

use crate::core::storage::base::{
    FileInfo, FileSystem, StorageClient, StorageSettings, StorageType,
};
use crate::core::storage::enums::{MultiPartUploader, StorageClientEnum};
use async_trait::async_trait;
use aws_smithy_types::byte_stream::ByteStream;
use aws_smithy_types::byte_stream::Length;

use futures::stream::TryStreamExt;
use futures::TryFutureExt;
use futures::TryStream;
use reqwest::multipart::{Form, Part};
use reqwest::{
    header::{self, HeaderMap, HeaderValue},
    Body, Client,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::sync::Mutex;
use tokio_util::codec::{BytesCodec, FramedRead};

const TIMEOUT_SECS: u64 = 30;
const CHUNK_SIZE: usize = 1024 * 1024 * 5;
const MAX_CHUNKS: u64 = 10000;

#[derive(Debug, Clone)]
pub enum RequestType {
    Get,
    Post,
    Put,
    Delete,
}

#[derive(Debug, Clone)]
pub enum Routes {
    Multipart,
    Presigned,
    List,
    ListInfo,
    Files,
}

impl Routes {
    pub fn as_str(&self) -> &str {
        match self {
            Routes::Files => "opsml/files",
            Routes::Multipart => "opsml/files/multipart",
            Routes::Presigned => "opsml/files/presigned",
            Routes::List => "opsml/files/list",
            Routes::ListInfo => "opsml/files/list_info",
        }
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
            RequestType::Delete => self
                .client
                .delete(format!("{}/{}", self.base_url, route.as_str()))
                .headers(headers)
                .query(&query_params)
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
    async fn create_multipart_upload(&mut self, rpath: &str) -> Result<String, ApiError> {
        let mut query_params = HashMap::new();
        query_params.insert("path".to_string(), rpath.to_string());

        let result = self
            .request_with_retry(
                Routes::Multipart,
                RequestType::Get,
                None,
                Some(query_params),
                None,
            )
            .await?;

        let session_url = result["session_url"]
            .as_str()
            .ok_or_else(|| ApiError::Error("Failed to get resumable id".to_string()))?;

        Ok(session_url.to_string())
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
    api_client: Arc<Mutex<ApiClient>>,
    pub bucket: PathBuf,
    storage_client: StorageClientEnum,
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
    fn storage_type(&self) -> StorageType {
        self.storage_client.storage_type()
    }
    async fn bucket(&self) -> &str {
        self.bucket.to_str().unwrap()
    }

    async fn new(settings: StorageSettings) -> Result<Self, StorageError> {
        let bucket = PathBuf::from(settings.storage_uri.clone());
        let api_kwargs = Self::get_http_kwargs(settings.kwargs.clone())?;

        let api_client = Arc::new(Mutex::new(
            ApiClient::new(
                api_kwargs.base_url,
                api_kwargs.use_auth,
                api_kwargs.path_prefix,
                api_kwargs.username,
                api_kwargs.password,
                api_kwargs.token,
            )
            .await
            .map_err(|e| StorageError::Error(format!("Failed to create api client: {}", e)))?,
        ));

        let storage_client = StorageClientEnum::new(settings.clone())
            .await
            .map_err(|e| StorageError::Error(format!("Failed to create storage client: {}", e)))?;

        // if setti

        Ok(Self {
            api_client,
            bucket,
            storage_client,
        })
    }

    async fn find(&self, path: &str) -> Result<Vec<String>, StorageError> {
        let mut params = HashMap::new();
        params.insert("path".to_string(), path.to_string());

        // Lock the Mutex to get mutable access to the ApiClient
        let mut api_client = self.api_client.lock().await;

        // need to clone because self is borrowed
        let response = api_client
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
        // Lock the Mutex to get mutable access to the ApiClient
        let mut api_client = self.api_client.lock().await;

        let response = api_client
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
        // Lock the Mutex to get mutable access to the ApiClient

        let mut api_client = self.api_client.lock().await;

        //let _response = api_client
        //    .download_to_file_with_retry(Routes::Multipart, local_path, remote_path)
        //    .await
        //    .map_err(|e| StorageError::Error(format!("Failed to download file: {}", e)))?;

        Ok(())
    }

    async fn copy_objects(&self, _source: &str, _destination: &str) -> Result<bool, StorageError> {
        unimplemented!();
    }

    async fn copy_object(&self, _source: &str, _destination: &str) -> Result<bool, StorageError> {
        unimplemented!();
    }

    async fn delete_object(&self, path: &str) -> Result<bool, StorageError> {
        // Lock the Mutex to get mutable access to the ApiClient
        let mut api_client = self.api_client.lock().await;

        let mut params = HashMap::new();
        params.insert("path".to_string(), path.to_string());
        params.insert("recursive".to_string(), "false".to_string());

        let _response = api_client
            .request_with_retry(Routes::Files, RequestType::Delete, None, Some(params), None)
            .await
            .map_err(|e| StorageError::Error(format!("Failed to delete file: {}", e)))?;

        Ok(true)
    }

    async fn delete_objects(&self, path: &str) -> Result<bool, StorageError> {
        // Lock the Mutex to get mutable access to the ApiClient
        let mut api_client = self.api_client.lock().await;

        let mut params = HashMap::new();
        params.insert("path".to_string(), path.to_string());
        params.insert("recursive".to_string(), "false".to_string());

        let _response = api_client
            .request_with_retry(Routes::Files, RequestType::Delete, None, Some(params), None)
            .await
            .map_err(|e| StorageError::Error(format!("Failed to delete file: {}", e)))?;

        Ok(true)
    }

    async fn generate_presigned_url(
        &self,
        __path: &str,
        __expiration: u64,
    ) -> Result<String, StorageError> {
        unimplemented!()
    }
}

impl HttpStorageClient {
    pub async fn generate_presigned_url_for_part(
        &self,
        path: &str,
        session_url: &str,
        part_number: u64,
    ) -> Result<String, StorageError> {
        let mut query_params = HashMap::new();
        query_params.insert("path".to_string(), path.to_string());
        query_params.insert("session_url".to_string(), session_url.to_string());
        query_params.insert("part_number".to_string(), part_number.to_string());
        query_params.insert("for_multi_part".to_string(), "true".to_string());

        // Lock the Mutex to get mutable access to the ApiClient
        let mut api_client = self.api_client.lock().await;

        let response = api_client
            .request_with_retry(
                Routes::Presigned,
                RequestType::Get,
                None,
                Some(query_params),
                None,
            )
            .await
            .map_err(|e| StorageError::Error(format!("Failed to generate presigned url: {}", e)))?;

        let url = &response["url"];

        Ok(url.to_string())
    }

    pub async fn create_multipart_upload(&self, path: &str) -> Result<String, StorageError> {
        // Lock the Mutex to get mutable access to the ApiClient
        let mut api_client = self.api_client.lock().await;

        let mut query_params = HashMap::new();
        query_params.insert("path".to_string(), path.to_string());

        let response = api_client
            .request_with_retry(
                Routes::Multipart,
                RequestType::Get,
                None,
                Some(query_params),
                None,
            )
            .await
            .map_err(|e| {
                StorageError::Error(format!("Failed to create multipart upload: {}", e))
            })?;

        let session_uri = &response["session_uri"];

        Ok(session_uri.to_string())
    }

    async fn upload_file_in_chunks(
        &self,
        lpath: &Path,
        rpath: &Path,
        uploader: &mut MultiPartUploader,
    ) -> Result<(), StorageError> {
        let file = File::open(lpath)
            .map_err(|e| StorageError::Error(format!("Failed to open file: {}", e)))
            .await?;

        // get file size
        let metadata = file
            .metadata()
            .map_err(|e| StorageError::Error(format!("Failed to get file metadata: {}", e)))
            .await?;

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

            let first_byte = chunk_index * chunk_size;
            let last_byte = first_byte + this_chunk - 1;

            let body = uploader
                .get_next_chunk(lpath, chunk_size, chunk_index, this_chunk)
                .await?;

            let part_number = (chunk_index as i32) + 1;

            let presigned_url = self
                .storage_client
                .generate_presigned_url_for_part(
                    part_number,
                    rpath.to_str().unwrap(),
                    uploader.session_url(),
                )
                .await?;

            uploader
                .upload_part_with_presigned_url(
                    &first_byte,
                    &last_byte,
                    &part_number,
                    &file_size,
                    body,
                    &presigned_url,
                )
                .await?;
        }

        uploader.complete_upload().await?;

        Ok(())
    }
}

pub struct HttpFSStorageClient {
    client: HttpStorageClient,
}

#[async_trait]
impl FileSystem<HttpStorageClient> for HttpFSStorageClient {
    fn name(&self) -> &str {
        "HttpFSStorageClient"
    }

    fn client(&self) -> &HttpStorageClient {
        &self.client
    }

    async fn new(settings: StorageSettings) -> Self {
        HttpFSStorageClient {
            client: HttpStorageClient::new(settings).await.unwrap(),
        }
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
}

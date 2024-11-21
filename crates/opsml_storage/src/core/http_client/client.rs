use crate::core::storage::base::FileInfo;
use crate::core::storage::enums::{MultiPartUploader, StorageClientEnum};
use crate::core::utils::error::{ApiError, StorageError};
use anyhow::{Context, Result as AnyhowResult};
use futures::TryFutureExt;
use opsml_settings::color::LogColors;
use opsml_settings::config::{ApiSettings, OpsmlStorageSettings, StorageType};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::File;

const TIMEOUT_SECS: u64 = 30;
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
    Healthcheck,
    StorageSettings,
}

impl Routes {
    pub fn as_str(&self) -> &str {
        match self {
            Routes::Files => "files",
            Routes::Multipart => "files/multipart",
            Routes::Presigned => "files/presigned",
            Routes::List => "files/list",
            Routes::ListInfo => "files/list/info",
            Routes::Healthcheck => "healthcheck",
            Routes::StorageSettings => "storage/settings",
        }
    }
}

/// Create a new HTTP client that can be shared across different clients
pub fn build_http_client(settings: &ApiSettings) -> Result<Client, ApiError> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "X-Prod-Token",
        HeaderValue::from_str(&settings.prod_token)
            .map_err(|e| ApiError::Error(format!("Failed to create header with error: {}", e)))?,
    );

    let client_builder = Client::builder().timeout(std::time::Duration::from_secs(TIMEOUT_SECS));
    let client = client_builder
        .default_headers(headers)
        .build()
        .map_err(|e| ApiError::Error(format!("Failed to create client with error: {}", e)))?;
    Ok(client)
}

#[derive(Debug, Clone)]
pub struct OpsmlApiClient {
    client: Client,
    settings: OpsmlStorageSettings,
    base_path: String,
}

impl OpsmlApiClient {
    pub async fn new(settings: &OpsmlStorageSettings, client: &Client) -> Result<Self, ApiError> {
        // setup headers

        let mut api_client = Self {
            client: client.clone(),
            settings: settings.clone(),
            base_path: format!(
                "{}/{}",
                settings.api_settings.base_url, settings.api_settings.opsml_dir
            ),
        };

        if settings.api_settings.use_auth {
            api_client.refresh_token().await?;
        }

        Ok(api_client)
    }

    async fn refresh_token(&mut self) -> Result<(), ApiError> {
        if !self.settings.api_settings.use_auth {
            return Ok(());
        }

        let form = json!({
            "username": self.settings.api_settings.username,
            "password": self.settings.api_settings.password,
        });

        let url = format!("{}/{}", self.base_path, "auth/token");
        let response = self
            .client
            .post(url)
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

        self.settings.api_settings.auth_token = token.to_string();

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
        let headers = headers.unwrap_or_default();

        let url = format!("{}/{}", self.base_path, route.as_str());
        let response = match request_type {
            RequestType::Get => self
                .client
                .get(url)
                .headers(headers)
                .query(&query_params)
                .bearer_auth(self.settings.api_settings.auth_token)
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!("Failed to send request with error: {}", e))
                })?,
            RequestType::Post => self
                .client
                .post(url)
                .headers(headers)
                .json(&body_params)
                .bearer_auth(self.settings.api_settings.auth_token)
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!("Failed to send request with error: {}", e))
                })?,
            RequestType::Put => self
                .client
                .put(url)
                .headers(headers)
                .json(&body_params)
                .bearer_auth(self.settings.api_settings.auth_token)
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!("Failed to send request with error: {}", e))
                })?,
            RequestType::Delete => self
                .client
                .delete(url)
                .headers(headers)
                .query(&query_params)
                .bearer_auth(self.settings.api_settings.auth_token)
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

        let query_params = query_params.unwrap_or_default();

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
}

pub struct HttpStorageClient {
    api_client: OpsmlApiClient,
    storage_client: StorageClientEnum,
}

impl HttpStorageClient {
    pub async fn new(settings: &mut OpsmlStorageSettings, client: &Client) -> AnyhowResult<Self> {
        let mut api_client = OpsmlApiClient::new(settings, client)
            .await
            .map_err(|e| StorageError::Error(format!("Failed to create api client: {}", e)))
            .context(LogColors::purple(
                "Error occurred while creating api client",
            ))?;

        // get storage type from opsml_server
        let storage_type =
            Self::get_storage_setting(&mut api_client)
                .await
                .context(LogColors::purple(
                    "Error occurred while getting storage type",
                ))?;

        // update settings type
        settings.storage_type = storage_type;

        // get storage client (options are gcs, aws, azure and local)
        let storage_client = StorageClientEnum::new(settings)
            .await
            .map_err(|e| StorageError::Error(format!("Failed to create storage client: {}", e)))
            .context(LogColors::green(
                "Error occurred while creating storage client",
            ))?;

        Ok(Self {
            api_client,
            storage_client,
        })
    }

    /// Function used to get the storage type from the server.
    /// A storage client is needed for when uploading and downloading files via presigned urls.
    ///
    /// # Arguments
    ///
    /// * `client` - The OpsmlApiClient
    ///
    /// # Returns
    ///
    /// * `StorageType` - The storage type
    async fn get_storage_setting(client: &mut OpsmlApiClient) -> Result<StorageType, ApiError> {
        let response = client
            .request_with_retry(Routes::StorageSettings, RequestType::Get, None, None, None)
            .await
            .map_err(|e| {
                ApiError::Error(LogColors::alert(&format!(
                    "Failed to get storage settings: {}",
                    e
                )))
            })?;

        let storage_type = response["storage_type"].to_string();

        StorageType::from_str(&storage_type).map_err(|e| {
            ApiError::Error(LogColors::alert(&format!(
                "Failed to get storage type: {}",
                e
            )))
        })
    }

    pub async fn find(&mut self, path: &str) -> Result<Vec<String>, StorageError> {
        let mut params = HashMap::new();
        params.insert("path".to_string(), path.to_string());

        // need to clone because self is borrowed
        let response = self
            .api_client
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

    pub async fn find_info(&mut self, path: &str) -> Result<Vec<FileInfo>, StorageError> {
        let mut params = HashMap::new();
        params.insert("path".to_string(), path.to_string());

        let response = self
            .api_client
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

    pub async fn get_object(
        &mut self,
        local_path: &str,
        remote_path: &str,
    ) -> Result<(), StorageError> {
        let _local_path = PathBuf::from(local_path);
        let _remote_path = PathBuf::from(remote_path);
        // Lock the Mutex to get mutable access to the ApiClient

        //let _response = api_client
        //    .download_to_file_with_retry(Routes::Multipart, local_path, remote_path)
        //    .await
        //    .map_err(|e| StorageError::Error(format!("Failed to download file: {}", e)))?;

        Ok(())
    }

    pub async fn delete_object(&mut self, path: &str) -> Result<bool, StorageError> {
        let mut params = HashMap::new();
        params.insert("path".to_string(), path.to_string());
        params.insert("recursive".to_string(), "false".to_string());

        let _response = self
            .api_client
            .request_with_retry(Routes::Files, RequestType::Delete, None, Some(params), None)
            .await
            .map_err(|e| StorageError::Error(format!("Failed to delete file: {}", e)))?;

        Ok(true)
    }

    pub async fn delete_objects(&mut self, path: &str) -> Result<bool, StorageError> {
        let mut params = HashMap::new();
        params.insert("path".to_string(), path.to_string());
        params.insert("recursive".to_string(), "true".to_string());

        let _response = self
            .api_client
            .request_with_retry(Routes::Files, RequestType::Delete, None, Some(params), None)
            .await
            .map_err(|e| StorageError::Error(format!("Failed to delete file: {}", e)))?;

        Ok(true)
    }

    pub async fn generate_presigned_url_for_part(
        &mut self,
        path: &str,
        session_url: &str,
        part_number: i32,
    ) -> Result<String, StorageError> {
        let mut query_params = HashMap::new();
        query_params.insert("path".to_string(), path.to_string());
        query_params.insert("session_url".to_string(), session_url.to_string());
        query_params.insert("part_number".to_string(), part_number.to_string());
        query_params.insert("for_multi_part".to_string(), "true".to_string());

        let response = self
            .api_client
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

    pub async fn create_multipart_upload(&mut self, path: &str) -> Result<String, StorageError> {
        let mut query_params = HashMap::new();
        query_params.insert("path".to_string(), path.to_string());

        let response = self
            .api_client
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

        let session_url = &response["session_url"];

        Ok(session_url.to_string())
    }

    /// Create a multipart uploader based on configured storage type
    pub async fn create_multipart_uploader(
        &mut self,
        rpath: &Path,
    ) -> Result<MultiPartUploader, StorageError> {
        let session_url = self
            .create_multipart_upload(rpath.to_str().unwrap())
            .await?;

        let uploader = self
            .storage_client
            .create_multipart_uploader(rpath, session_url)
            .await?;

        Ok(uploader)
    }

    pub async fn upload_file_in_chunks(
        &mut self,
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

            // call server to get presigned url for part
            let presigned_url = self
                .generate_presigned_url_for_part(
                    rpath.to_str().unwrap(),
                    &uploader.session_url(),
                    part_number,
                )
                .await?;

            // pass presigned url to upload part
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

    pub async fn generate_presigned_url(&mut self, path: &str) -> Result<String, StorageError> {
        let mut query_params = HashMap::new();
        query_params.insert("path".to_string(), path.to_string());

        let response = self
            .api_client
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::{Server, ServerGuard};
    use opsml_settings::config::OpsmlConfig;
    use tokio;

    async fn setup_server() -> (ServerGuard, String) {
        let server = Server::new_async().await;
        let server_url = server.url();
        (server, server_url)
    }

    async fn setup_client(server_url: String, use_auth: Option<bool>) -> OpsmlApiClient {
        let config = OpsmlConfig::new();
        let mut settings = config.storage_settings();

        // set up some auth
        settings.api_settings.username = "username".to_string();
        settings.api_settings.password = "password".to_string();
        settings.api_settings.use_auth = use_auth.unwrap_or(false);
        settings.api_settings.base_url = server_url.to_string();

        let client = build_http_client(&settings.api_settings).unwrap();
        OpsmlApiClient::new(&settings, &client).await.unwrap()
    }

    #[tokio::test]
    async fn test_api_client_no_auth() {
        let (mut server, server_url) = setup_server().await;
        let mut api_client = setup_client(server_url, None).await;

        let _mock = server
            .mock("GET", "/opsml/files")
            .with_status(200)
            .with_body(r#"{"status": "ok"}"#)
            .create();

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

        let mut api_client = setup_client(server_url, Some(true)).await;

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

        let mut api_client = setup_client(server_url, Some(true)).await;

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

        let mut api_client = setup_client(server_url, Some(true)).await;
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

        let mut api_client = setup_client(server_url, Some(true)).await;

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

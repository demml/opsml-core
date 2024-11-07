use crate::core::utils::error::ApiError;

use pyo3::prelude::*;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

const PATH_PREFIX: &str = "opsml";
const TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub enum RequestType {
    Get,
    Post,
    Put,
    StreamPost,
    StreamGet,
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
                    ApiError::Error(format!(
                        "Failed to create header with error: {}",
                        e.to_string()
                    ))
                })?,
            );
        }
        let client = client_builder
            .default_headers(headers)
            .build()
            .map_err(|e| {
                ApiError::Error(format!(
                    "Failed to create client with error: {}",
                    e.to_string()
                ))
            })?;

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
            .map_err(|e| {
                ApiError::Error(format!(
                    "Failed to send request with error: {}",
                    e.to_string()
                ))
            })?
            .json::<Value>()
            .await
            .map_err(|e| {
                ApiError::Error(format!(
                    "Failed to parse response with error: {}",
                    e.to_string()
                ))
            })?;

        // check if the response has "access_token" field
        let token = response["access_token"]
            .as_str()
            .ok_or_else(|| ApiError::Error("Failed to get access token".to_string()))?;

        self.auth_token = Some(token.to_string());

        Ok(())
    }

    async fn request(
        self,
        route: &str,
        request_type: RequestType,
        params: Option<Value>,
    ) -> Result<Value, ApiError> {
        let response = match request_type {
            RequestType::Get => self
                .client
                .get(format!("{}/{}", self.base_url, route))
                // add bearer token if it exists
                .header(
                    "Authorization",
                    format!("Bearer {}", self.auth_token.unwrap_or("".to_string())),
                )
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!(
                        "Failed to send request with error: {}",
                        e.to_string()
                    ))
                })?,
            RequestType::Post => self
                .client
                .post(format!("{}/{}", self.base_url, route))
                .header(
                    "Authorization",
                    format!("Bearer {}", self.auth_token.unwrap_or("".to_string())),
                )
                .json(&params)
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!(
                        "Failed to send request with error: {}",
                        e.to_string()
                    ))
                })?,
            RequestType::Put => self
                .client
                .put(format!("{}/{}", self.base_url, route))
                .header(
                    "Authorization",
                    format!("Bearer {}", self.auth_token.unwrap_or("".to_string())),
                )
                .json(&params)
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!(
                        "Failed to send request with error: {}",
                        e.to_string()
                    ))
                })?,
            RequestType::StreamPost => self
                .client
                .post(format!("{}/{}", self.base_url, route))
                .header(
                    "Authorization",
                    format!("Bearer {}", self.auth_token.unwrap_or("".to_string())),
                )
                .json(&params)
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!(
                        "Failed to send request with error: {}",
                        e.to_string()
                    ))
                })?,
            RequestType::StreamGet => self
                .client
                .get(format!("{}/{}", self.base_url, route))
                .header(
                    "Authorization",
                    format!("Bearer {}", self.auth_token.unwrap_or("".to_string())),
                )
                .send()
                .await
                .map_err(|e| {
                    ApiError::Error(format!(
                        "Failed to send request with error: {}",
                        e.to_string()
                    ))
                })?,
        };

        let response = response.json::<Value>().await.map_err(|e| {
            ApiError::Error(format!(
                "Failed to parse response with error: {}",
                e.to_string()
            ))
        })?;

        Ok(response)
    }

    pub async fn request_with_retry(
        &mut self,
        route: &str,
        request_type: RequestType,
        params: Option<Value>,
    ) -> Result<Value, ApiError> {
        // this will attempt to send a request. If the request fails, it will refresh the token and try again. If it fails all 3 times it will return an error
        let mut attempts = 0;
        let max_attempts = 3;
        let mut response: Result<Value, ApiError>;

        loop {
            let client = self.clone();
            response = client
                .request(route, request_type.clone(), params.clone())
                .await;

            if response.is_ok() || attempts >= max_attempts {
                break;
            }

            if let Err(ApiError::Error(ref e)) = response {
                if e.contains("401") {
                    self.refresh_token().await?;
                }
            }

            attempts += 1;
        }

        let response = response.map_err(|e| {
            ApiError::Error(format!(
                "Failed to send request with error: {}",
                e.to_string()
            ))
        })?;

        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use tokio;

    #[tokio::test]
    async fn test_api_client_no_auth() {
        let mut server = Server::new_async().await;
        let server_url = server.url();

        let _mock = server.mock("GET", "/opsml/test").with_status(200).create();

        let api_client =
            ApiClient::new(server_url, false, PATH_PREFIX.to_string(), None, None, None)
                .await
                .unwrap();

        let response = api_client
            .client
            .get(format!("{}/test", api_client.base_url))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }

    #[tokio::test]
    async fn test_api_client_with_auth() {
        let mut server = Server::new_async().await;
        let server_url = server.url();

        let _token_mock = server
            .mock("POST", "/opsml/auth/token")
            .with_status(200)
            .with_body(r#"{"access_token": "test_token"}"#)
            .create();

        let api_client = ApiClient::new(
            server_url,
            true,
            PATH_PREFIX.to_string(),
            Some("username".to_string()),
            Some("password".to_string()),
            None,
        )
        .await
        .unwrap();

        let _auth_mock = server
            .mock("GET", "/opsml/test")
            .match_header("Authorization", "Bearer test_token")
            .with_status(200)
            .create();

        let response = api_client
            .client
            .get(format!("{}/test", api_client.base_url))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
    }
}

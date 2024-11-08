#[cfg(feature = "google_storage")]
pub mod google_storage {

    use crate::core::utils::error::GoogleStorageError;
    use base64::prelude::*;
    use bytes::Bytes;
    use futures::stream::Stream;
    use futures::TryStreamExt;
    use google_cloud_auth::credentials::CredentialsFile;
    use google_cloud_storage::client::{Client, ClientConfig};
    use google_cloud_storage::http::objects::download::Range;
    use google_cloud_storage::http::objects::get::GetObjectRequest;
    use serde_json::Value;
    use std::{env, path::PathBuf};

    #[derive(Clone)]
    pub struct GcpCreds {
        pub creds: Option<CredentialsFile>,
        pub project: Option<String>,
        pub default_creds: bool,
        pub service_account: Option<Value>,
    }

    impl GcpCreds {
        pub async fn new() -> Result<Self, GoogleStorageError> {
            let mut creds = GcpCreds {
                creds: None,
                project: None,
                default_creds: false,
                service_account: None,
            };
            creds
                .check_model()
                .await
                .map_err(|e| GoogleStorageError::Error(format!("Unable to check model: {}", e)))?;
            Ok(creds)
        }

        async fn check_model(&mut self) -> Result<(), GoogleStorageError> {
            if let Ok(base64_creds) = env::var("GOOGLE_ACCOUNT_JSON_BASE64") {
                let cred_string = self.decode_base64(&base64_creds).map_err(|e| {
                    GoogleStorageError::Error(format!("Unable to decode base64: {}", e))
                })?;

                self.creds = Some(CredentialsFile::new_from_str(&cred_string).await.map_err(
                    |e| {
                        GoogleStorageError::Error(format!(
                            "Unable to create credentials file from string: {}",
                            e
                        ))
                    },
                )?);
            }

            if let Ok(_service_account_file) = env::var("GOOGLE_APPLICATION_CREDENTIALS_JSON") {
                self.creds = Some(CredentialsFile::new().await.map_err(|e| {
                    GoogleStorageError::Error(format!(
                        "Unable to create credentials file from file: {}",
                        e
                    ))
                })?);
            }

            Ok(())
        }

        fn decode_base64(
            &mut self,
            service_base64_creds: &str,
        ) -> Result<String, GoogleStorageError> {
            let decoded = BASE64_STANDARD.decode(service_base64_creds).map_err(|e| {
                GoogleStorageError::Error(format!("Unable to decode base64: {}", e))
            })?;
            let decoded_str = String::from_utf8(decoded).map_err(|e| {
                GoogleStorageError::Error(format!("Unable to convert to string: {}", e))
            })?;
            Ok(decoded_str)
        }
    }

    pub struct GoogleStorageClient {
        pub client: Client,
        creds: GcpCreds,
        pub bucket: String,
    }

    impl GoogleStorageClient {
        pub async fn new(bucket: String) -> Result<Self, GoogleStorageError> {
            let creds = GcpCreds::new().await?;
            let client = GoogleStorageClient::build_client(creds.clone()).await?;

            Ok(GoogleStorageClient {
                client,
                creds,
                bucket,
            })
        }

        pub async fn build_client(creds: GcpCreds) -> Result<Client, GoogleStorageError> {
            // If no credentials, attempt to create a default client pulling from the environment
            if creds.creds.is_none() {
                let config = ClientConfig::default().with_auth().await;

                // if error, use ClientConfig::default().anonymous();
                let config = match config {
                    Ok(config) => config,
                    Err(_) => ClientConfig::default().anonymous(),
                };

                return Ok(Client::new(config));

            // if creds are set (base64 for JSON file)
            } else {
                let config = ClientConfig::default()
                    .with_credentials(creds.creds.unwrap())
                    .await
                    .map_err(|e| {
                        GoogleStorageError::Error(format!(
                            "Unable to create client with credentials: {}",
                            e
                        ))
                    })?;

                let client = Client::new(config);
                return Ok(client);
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
        async fn get_object_stream(
            &self,
            rpath: &str,
        ) -> Result<impl Stream<Item = Result<Bytes, GoogleStorageError>>, GoogleStorageError>
        {
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
                    GoogleStorageError::Error(format!("Unable to download object: {}", e))
                })?;

            Ok(result.map_err(|e| {
                GoogleStorageError::Error(format!("Error while streaming object: {}", e))
            }))

            //
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

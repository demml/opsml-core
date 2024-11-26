use crate::storage::base::StorageClient;
use async_trait::async_trait;
use azure_core::date::iso8601::option;
use azure_identity::{EnvironmentCredential, TokenCredentialOptions};
use azure_storage::prelude::*;
use azure_storage::shared_access_signature::account_sas::{
    AccountSasPermissions, AccountSasResourceType,
};
use azure_storage_blobs::prelude::*;
use futures::stream::StreamExt;
use opsml_constants::{DOWNLOAD_CHUNK_SIZE, UPLOAD_CHUNK_SIZE};
use opsml_error::error::StorageError;
use opsml_settings::config::OpsmlStorageSettings;
use opsml_settings::config::StorageType;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use time::OffsetDateTime;

pub struct AzureCreds {
    pub account: String,
    pub creds: StorageCredentials,
}

impl AzureCreds {
    pub async fn new() -> Result<Self, StorageError> {
        let credential = azure_identity::create_credential().map_err(|e| {
            StorageError::Error(format!("Failed to create Azure credential: {:?}", e))
        })?;

        let account = env::var("AZURE_STORAGE_ACCOUNT").map_err(|e| {
            StorageError::Error(format!("Failed to get Azure storage account: {:?}", e))
        })?;

        let creds = StorageCredentials::token_credential(credential);

        Ok(Self { account, creds })
    }
}

#[derive(Clone)]
pub struct AzureStorageClient {
    pub client: BlobServiceClient,
    pub bucket: String,
}

#[async_trait]
impl StorageClient for AzureStorageClient {
    fn storage_type(&self) -> StorageType {
        StorageType::Azure
    }

    async fn bucket(&self) -> &str {
        &self.bucket
    }

    async fn new(settings: &OpsmlStorageSettings) -> Result<Self, StorageError> {
        // Get Azure credentials (anonymous if client mode, else use AzureCreds)
        let creds = match settings.client_mode {
            false => AzureCreds::new().await?,
            true => AzureCreds {
                account: "anonymous".to_string(),
                creds: StorageCredentials::anonymous(),
            },
        };

        let client = BlobServiceClient::new(creds.account, creds.creds);
        let bucket = settings
            .storage_uri
            .strip_prefix("azure://")
            .unwrap_or(&settings.storage_uri)
            .to_string();

        Ok(Self { client, bucket })
    }

    async fn get_object(&self, lpath: &str, rpath: &str) -> Result<(), StorageError> {
        let lpath = Path::new(lpath);
        let rpath = Path::new(rpath);

        if lpath.extension().is_none() || rpath.extension().is_none() {
            return Err(StorageError::Error(
                "Local and remote paths must have suffixes".to_string(),
            ));
        }

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

        let container = self.client.container_client(self.bucket.as_str());
        let blob = container.blob_client(rpath.to_str().unwrap());

        let mut stream = blob
            .get()
            .chunk_size(DOWNLOAD_CHUNK_SIZE as u64)
            .into_stream();

        // iterate over the stream and write to the file
        while let Some(value) = stream.next().await {
            let chunk = value
                .map_err(|e| StorageError::Error(format!("Error: {}", e)))?
                .data;

            // collect into bytes
            let bytes = chunk
                .collect()
                .await
                .map_err(|e| StorageError::Error(format!("Error: {}", e)))?;

            file.write_all(&bytes)
                .map_err(|e| StorageError::Error(format!("Unable to write to file: {}", e)))?;
        }

        Ok(())
    }

    async fn generate_presigned_url(
        &self,
        path: &str,
        expiration: u64,
    ) -> Result<String, StorageError> {
        let container = self.client.container_client(self.bucket.as_str());
        let blob = container.blob_client(path);

        let expiry = OffsetDateTime::now_utc() + expiration;
        let permissions = AccountSasPermissions {
            read: true,
            write: false,
            delete: false,
            list: false,
            add: false,
            create: false,
            update: false,
            process: false,
        };
        let signature = self
            .client
            .shared_access_signature(AccountSasResourceType::Object, expiry, permissions)
            .await
            .map_err(|e| StorageError::Error(format!("Error: {}", e)))?;

        let url = blob
            .generate_signed_blob_url(&signature)
            .map_err(|e| StorageError::Error(format!("Failed to generate presigned url: {}", e)))?;

        Ok(url.to_string())
    }
}

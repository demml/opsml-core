use opsml_core::core::storage::base::{ClientType, ResumableClient, StorageClient};
use opsml_core::core::utils::error::StorageError;
use pyo3::prelude::*;
use tokio::runtime::Runtime;

#[pyclass]
pub struct OpsmlStorageClient {
    storage_client: StorageClient,
    resumable_client: Option<ResumableClient>,
}

#[pymethods]
impl OpsmlStorageClient {
    #[new]
    pub fn new(client_type: ClientType, bucket: String) -> Self {
        let storage_client = Runtime::new()
            .unwrap()
            .block_on(StorageClient::new(client_type, bucket))
            .unwrap();

        // Get the future

        OpsmlStorageClient {
            storage_client,
            resumable_client: None,
        }
    }

    pub async fn find(&self, path: String) -> Vec<String> {
        self.storage_client.find(&path).await.unwrap()
    }

    pub async fn create_resumable_upload_session(
        &mut self,
        path: String,
        chunk_size: u64,
        total_size: u64,
    ) -> Result<(), StorageError> {
        let resumable_client = self
            .storage_client
            .create_resumable_upload_session(&path, chunk_size, total_size)
            .await?;

        self.resumable_client = Some(resumable_client);
        Ok(())
    }

    pub async fn upload_chunk(&mut self, data: Vec<u8>) -> Result<Option<String>, StorageError> {
        if let Some(ref mut resumable_client) = self.resumable_client {
            resumable_client.upload_multiple_chunks(data.into()).await
        } else {
            Err(StorageError::Error("No resumable client found".to_string()))
        }
    }
}

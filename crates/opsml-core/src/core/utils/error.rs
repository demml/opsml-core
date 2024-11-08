use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Api client failure: {0}")]
    Error(String),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage client failure: {0}")]
    Error(String),

    #[error("Storage client failure. Unsupported client.")]
    UnsupportedClient,
}

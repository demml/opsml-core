use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Api client failure: {0}")]
    Error(String),
}

use thiserror::Error;

#[derive(Error, Debug, serde::Serialize)]
pub enum ServerError {
    #[error("Server failure: {0}")]
    Error(String),
}

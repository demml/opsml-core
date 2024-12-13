use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct AuthError {
    pub error: String,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct JwtToken {
    pub token: String,
}

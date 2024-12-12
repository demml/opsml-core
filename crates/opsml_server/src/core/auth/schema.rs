#[derive(serde::Serialize)]
pub struct AuthError {
    pub error: String,
    pub message: String,
}

#[derive(serde::Serialize)]
pub struct JwtToken {
    pub token: String,
}

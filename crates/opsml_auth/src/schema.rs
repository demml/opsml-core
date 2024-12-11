use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub created_at: NaiveDateTime,
    pub active: bool,
    pub username: String,
    pub password_hash: String,
    pub permissions: Vec<String>,
    pub group_permissions: Vec<String>,
}

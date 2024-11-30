use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VersionResult {
    pub date: String,
    pub timestamp: i64,
    pub name: String,
    pub repository: String,
    pub version: String,
}

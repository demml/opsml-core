use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use sqlx::sqlite::SqliteRow;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionResult {
    pub date: NaiveDateTime,
    pub timestamp: i64,
    pub name: String,
    pub repository: String,
    pub version: String,
}

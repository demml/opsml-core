use chrono::{NaiveDateTime, Timelike};
use opsml_error::error::UtilError;
use uuid::Uuid;

/// Check if a string is a valid UUIDv4
///
/// # Arguments
///
/// * `uid` - A string slice that holds the UUID
///
/// # Returns
///
/// * `bool` - A boolean indicating if the UUID is valid
pub fn is_valid_uuid4(uid: &str) -> Result<bool, UtilError> {
    match Uuid::parse_str(uid) {
        Ok(uuid) => Ok(uuid.get_version_num() == 4),
        Err(_) => Err(UtilError::UuidError),
    }
}

pub fn get_epoch_time_to_search(max_date: &str) -> Result<i64, UtilError> {
    const YEAR_MONTH_DATE: &str = "%Y-%m-%d";

    // Parse the date string into a NaiveDateTime
    let converted_date = NaiveDateTime::parse_from_str(max_date, YEAR_MONTH_DATE)
        .map_err(|_| UtilError::DateError)?;

    // Replace hour, minute, and second to get the max values for the date
    let max_date = converted_date
        .with_hour(23)
        .unwrap()
        .with_minute(59)
        .unwrap()
        .with_second(59)
        .unwrap();

    // Convert NaiveDateTime to timestamp in microseconds
    let timestamp = max_date.and_utc().timestamp() * 1_000_000;

    Ok(timestamp)
}

pub fn get_utc_date() -> String {
    chrono::Utc::now().format("%Y-%m-%d").to_string()
}

pub fn get_utc_timestamp() -> i64 {
    chrono::Utc::now().timestamp()
}

pub fn get_utc_datetime() -> NaiveDateTime {
    chrono::Utc::now().naive_utc()
}

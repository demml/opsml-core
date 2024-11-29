use std::io;

use opsml_error::error::LoggingError;
use tracing_subscriber;
use tracing_subscriber::fmt::time::UtcTime;

const DEFAULT_TIME_PATTERN: &str =
    "[year]-[month]-[day]T[hour repr:24]:[minute]:[second]::[subsecond digits:4]";

pub async fn setup_logging() -> Result<(), LoggingError> {
    let time_format = time::format_description::parse(DEFAULT_TIME_PATTERN).map_err(|e| {
        LoggingError::Error(format!(
            "Failed to parse time format: {} with error: {}",
            DEFAULT_TIME_PATTERN, e
        ))
    })?;

    tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .flatten_event(true)
        .with_thread_ids(true)
        .with_timer(UtcTime::new(time_format))
        .with_writer(io::stdout)
        .try_init()
        .map_err(|e| LoggingError::Error(format!("Failed to setup logging with error: {}", e)))?;

    Ok(())
}

use crate::core::files::schema::{ResumableArgs, ResumableId};
use crate::core::state::AppState;
use axum::extract::Query;
/// Route for debugging information
use axum::extract::State;
use std::sync::Arc;

pub async fn create_resumable_upload(
    State(data): State<Arc<AppState>>,
    params: Query<ResumableArgs>,
) -> Result<ResumableId> {
    let query_result = &data.db.get_drift_alerts(&params).await;
}

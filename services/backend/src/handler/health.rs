use std::sync::Arc;

use axum::{Json, extract::State};

use crate::handler::response::HealthResponse;
use crate::usecase::check_health::CheckHealthUsecase;

/// `GET /api/health`
///
/// Always returns HTTP 200. Inspect `db_connected` to detect degraded state.
#[tracing::instrument(skip(usecase), fields(endpoint = "health"))]
pub async fn health(State(usecase): State<Arc<CheckHealthUsecase>>) -> Json<HealthResponse> {
    let status = usecase.execute().await;
    tracing::info!(
        status = status.status,
        db_connected = status.db_connected,
        reinfolib_key_set = status.reinfolib_key_set,
        "health check complete"
    );
    Json(HealthResponse::from(status))
}

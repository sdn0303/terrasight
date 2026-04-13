//! `GET /api/v1/health` handler.
//!
//! Liveness and readiness probe used by Docker / Kubernetes and the
//! Terrasight frontend status bar. Always returns `200 OK` so that the
//! load balancer keeps the instance in rotation; the response body carries
//! `db_connected` and `reinfolib_key_set` flags for degraded-state
//! detection.

use std::sync::Arc;

use axum::{Json, extract::State};

use crate::handler::response::HealthResponse;
use crate::usecase::check_health::CheckHealthUsecase;

/// Handles `GET /api/v1/health`.
///
/// Always returns `200 OK`. The response body is a [`HealthResponse`]
/// containing `status`, `db_connected`, `reinfolib_key_set`, and the
/// API `version` string. Callers should treat `db_connected = false` as
/// a degraded state even though the HTTP status is `200`.
#[tracing::instrument(skip(usecase), fields(endpoint = "health"))]
pub(crate) async fn health(State(usecase): State<Arc<CheckHealthUsecase>>) -> Json<HealthResponse> {
    let status = usecase.execute().await;
    tracing::info!(
        status = status.status,
        db_connected = status.db_connected,
        reinfolib_key_set = status.reinfolib_key_set,
        "health check complete"
    );
    Json(HealthResponse::from(status))
}

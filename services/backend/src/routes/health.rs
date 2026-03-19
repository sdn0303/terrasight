use axum::{extract::State, Json};
use serde::Serialize;

use crate::models::AppState;

/// Response body for `GET /api/health`.
#[derive(Serialize)]
pub struct HealthResponse {
    /// `"ok"` when PostGIS is reachable, `"degraded"` otherwise.
    pub status: String,
    /// Whether a `SELECT 1` against PostGIS succeeded.
    pub db_connected: bool,
    /// Whether `REINFOLIB_API_KEY` env var is set.
    pub reinfolib_key_set: bool,
    /// Crate version from `Cargo.toml`.
    pub version: String,
}

/// Health check handler.
///
/// Always returns HTTP 200. Inspect `db_connected` in the body to detect degraded state.
pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    let db_connected = sqlx::query("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    Json(HealthResponse {
        status: if db_connected {
            "ok".to_string()
        } else {
            "degraded".to_string()
        },
        db_connected,
        reinfolib_key_set: state.reinfolib_key.is_some(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

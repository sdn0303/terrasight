//! Response DTO for `GET /api/health`.

use serde::Serialize;

use crate::domain::entity::HealthStatus;

/// Response for `GET /api/v1/health`.
///
/// The HTTP status is always `200 OK`. Callers should check `db_connected`
/// to detect a degraded but still-serving state.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    /// Overall status string. Always `"ok"` (HTTP 200 is returned even in
    /// degraded mode; inspect the boolean flags for actual health).
    pub status: &'static str,
    /// `true` when a test query against the PostgreSQL connection pool succeeded.
    pub db_connected: bool,
    /// `true` when the `REINFOLIB_API_KEY` environment variable is set.
    pub reinfolib_key_set: bool,
    /// Semantic version string of the running API binary (from `CARGO_PKG_VERSION`).
    pub version: &'static str,
}

impl From<HealthStatus> for HealthResponse {
    fn from(h: HealthStatus) -> Self {
        Self {
            status: h.status,
            db_connected: h.db_connected,
            reinfolib_key_set: h.reinfolib_key_set,
            version: h.version,
        }
    }
}

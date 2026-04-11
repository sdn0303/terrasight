//! Response DTO for `GET /api/health`.

use serde::Serialize;

use crate::domain::entity::HealthStatus;

/// Response for `GET /api/health`.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub db_connected: bool,
    pub reinfolib_key_set: bool,
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

//! Health check domain type for the `/api/v1/health` endpoint.

/// Health check result for the `/api/v1/health` endpoint.
///
/// Moved to the domain layer (P0 polish) so the handler does not need to
/// import from the usecase layer.
#[derive(Debug, Clone)]
pub struct HealthStatus {
    /// Overall status string: `"ok"` or `"degraded"`.
    ///
    /// See `constants::HEALTH_STATUS_OK` and `constants::HEALTH_STATUS_DEGRADED`.
    pub status: &'static str,
    /// `true` when the PostgreSQL `SELECT 1` probe succeeded.
    pub db_connected: bool,
    /// `true` when the `REINFOLIB_API_KEY` environment variable is set.
    pub reinfolib_key_set: bool,
    /// Crate version string from `CARGO_PKG_VERSION`.
    pub version: &'static str,
}

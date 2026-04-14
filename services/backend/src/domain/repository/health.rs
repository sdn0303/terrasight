//! [`HealthRepository`] trait — database connectivity probe.

use async_trait::async_trait;

/// Repository for database health probes.
///
/// Implemented by `PgHealthRepository` in the `infra` layer.
#[async_trait]
pub trait HealthRepository: Send + Sync {
    /// Check database connectivity (`SELECT 1`).
    ///
    /// Returns `true` when the database is reachable, `false` otherwise.
    /// This method is infallible by design — callers treat `false` as a
    /// degraded-health signal rather than an error.
    async fn check_connection(&self) -> bool;
}

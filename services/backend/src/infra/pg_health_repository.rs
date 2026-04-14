//! PostgreSQL implementation of [`HealthRepository`].
//!
//! Implements [`HealthRepository`](crate::domain::repository::HealthRepository)
//! for the `/api/v1/health` liveness check. Issues a minimal `SELECT 1`
//! query through the connection pool, wrapped in a [`HEALTH_CHECK_TIMEOUT`]
//! deadline so that a hung database does not block the health endpoint.

use std::time::Duration;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::repository::HealthRepository;

/// Maximum time to wait for the health check query before declaring degraded.
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(3);

/// PostgreSQL implementation of [`HealthRepository`](crate::domain::repository::HealthRepository).
pub(crate) struct PgHealthRepository {
    pool: PgPool,
}

impl PgHealthRepository {
    /// Create a new repository backed by the given connection pool.
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthRepository for PgHealthRepository {
    /// Return `true` if the database is reachable within [`HEALTH_CHECK_TIMEOUT`].
    ///
    /// Logs a `warn`-level message on failure (timeout or query error) so
    /// that alerting can pick up degraded state from the structured log stream.
    async fn check_connection(&self) -> bool {
        let result = tokio::time::timeout(
            HEALTH_CHECK_TIMEOUT,
            sqlx::query("SELECT 1").fetch_one(&self.pool),
        )
        .await;

        let ok = matches!(result, Ok(Ok(_)));
        if !ok {
            tracing::warn!(
                timed_out = result.is_err(),
                "health_check query failed or timed out"
            );
        }
        tracing::debug!(db_connected = ok, "health_check query");
        ok
    }
}

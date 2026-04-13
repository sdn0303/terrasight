//! Usecase: liveness and readiness health check.
//!
//! Orchestrates [`HealthRepository::check_connection`] and combines the result
//! with static configuration flags (reinfolib API key presence) into a
//! [`HealthStatus`] response. Called by `GET /api/v1/health`.

use std::sync::Arc;

use crate::domain::constants::{HEALTH_STATUS_DEGRADED, HEALTH_STATUS_OK};
use crate::domain::entity::HealthStatus;
use crate::domain::repository::HealthRepository;

/// Usecase for `GET /api/v1/health`.
pub(crate) struct CheckHealthUsecase {
    health_repo: Arc<dyn HealthRepository>,
    reinfolib_key_set: bool,
}

impl CheckHealthUsecase {
    /// Construct the usecase.
    ///
    /// `reinfolib_key_set` reflects whether `REINFOLIB_API_KEY` was configured
    /// at startup; it is included in the health response so operators can
    /// distinguish the PostGIS-fallback path from the live-API path.
    pub(crate) fn new(health_repo: Arc<dyn HealthRepository>, reinfolib_key_set: bool) -> Self {
        Self {
            health_repo,
            reinfolib_key_set,
        }
    }

    /// Check database connectivity and return a [`HealthStatus`].
    ///
    /// Returns `"ok"` when the database responds within the health-check timeout,
    /// or `"degraded"` otherwise. This method never fails — a degraded status is
    /// always returned rather than propagating the underlying error.
    #[tracing::instrument(skip(self), fields(usecase = "check_health"))]
    pub(crate) async fn execute(&self) -> HealthStatus {
        let db_connected = self.health_repo.check_connection().await;

        if !db_connected {
            tracing::error!(db_connected = false, "database health check failed");
        }

        HealthStatus {
            status: match db_connected {
                true => HEALTH_STATUS_OK,
                false => HEALTH_STATUS_DEGRADED,
            },
            db_connected,
            reinfolib_key_set: self.reinfolib_key_set,
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::mock::MockHealthRepository;

    #[tokio::test]
    async fn execute_happy_path_reports_ok_when_db_connected() {
        let repo = Arc::new(MockHealthRepository::new().with_check_connection(true));
        let usecase = CheckHealthUsecase::new(repo, true);

        let status = usecase.execute().await;

        assert_eq!(status.status, HEALTH_STATUS_OK);
        assert!(status.db_connected);
        assert!(status.reinfolib_key_set);
    }

    #[tokio::test]
    async fn execute_reports_degraded_when_db_disconnected() {
        let repo = Arc::new(MockHealthRepository::new().with_check_connection(false));
        let usecase = CheckHealthUsecase::new(repo, false);

        let status = usecase.execute().await;

        assert_eq!(status.status, HEALTH_STATUS_DEGRADED);
        assert!(!status.db_connected);
        assert!(!status.reinfolib_key_set);
    }
}

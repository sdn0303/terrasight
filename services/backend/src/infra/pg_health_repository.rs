use std::time::Duration;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::repository::HealthRepository;

/// Maximum time to wait for the health check query before declaring degraded.
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(3);

pub(crate) struct PgHealthRepository {
    pool: PgPool,
}

impl PgHealthRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthRepository for PgHealthRepository {
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

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::repository::HealthRepository;

pub struct PgHealthRepository {
    pool: PgPool,
}

impl PgHealthRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HealthRepository for PgHealthRepository {
    async fn check_connection(&self) -> bool {
        let ok = sqlx::query("SELECT 1").fetch_one(&self.pool).await.is_ok();
        tracing::debug!(db_connected = ok, "health_check query");
        ok
    }
}

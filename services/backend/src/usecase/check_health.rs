use std::sync::Arc;

use crate::domain::constants::{HEALTH_STATUS_DEGRADED, HEALTH_STATUS_OK};
use crate::domain::entity::HealthStatus;
use crate::domain::repository::HealthRepository;

pub struct CheckHealthUsecase {
    health_repo: Arc<dyn HealthRepository>,
    reinfolib_key_set: bool,
}

impl CheckHealthUsecase {
    pub fn new(health_repo: Arc<dyn HealthRepository>, reinfolib_key_set: bool) -> Self {
        Self {
            health_repo,
            reinfolib_key_set,
        }
    }

    pub async fn execute(&self) -> HealthStatus {
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

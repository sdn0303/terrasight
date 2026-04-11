use std::sync::Arc;

use crate::domain::entity::AdminAreaStats;
use crate::domain::error::DomainError;
use crate::domain::repository::AdminAreaStatsRepository;
use crate::domain::value_object::AreaCode;

/// Usecase: fetch aggregated statistics for an administrative area.
pub struct GetAreaStatsUsecase {
    repo: Arc<dyn AdminAreaStatsRepository>,
}

impl GetAreaStatsUsecase {
    pub fn new(repo: Arc<dyn AdminAreaStatsRepository>) -> Self {
        Self { repo }
    }

    /// Execute the area-stats query for the given administrative area code.
    pub async fn execute(&self, code: &AreaCode) -> Result<AdminAreaStats, DomainError> {
        self.repo.get_area_stats(code).await
    }
}

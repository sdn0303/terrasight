use std::sync::Arc;

use crate::domain::entity::AdminAreaStats;
use crate::domain::error::DomainError;
use crate::domain::repository::AdminAreaStatsRepository;

/// Usecase: fetch aggregated statistics for an administrative area.
pub struct GetAreaStatsUsecase {
    repo: Arc<dyn AdminAreaStatsRepository>,
}

impl GetAreaStatsUsecase {
    pub fn new(repo: Arc<dyn AdminAreaStatsRepository>) -> Self {
        Self { repo }
    }

    /// Execute the area-stats query for the given administrative area code.
    ///
    /// Returns [`DomainError::MissingParameter`] when `code` is empty so that the
    /// handler can surface a `400 Bad Request` without touching the database.
    pub async fn execute(&self, code: &str) -> Result<AdminAreaStats, DomainError> {
        if code.is_empty() {
            return Err(DomainError::MissingParameter("code".into()));
        }
        self.repo.get_area_stats(code).await
    }
}

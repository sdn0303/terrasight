//! Usecase: fetch aggregated statistics for an administrative area code.
//!
//! Delegates to [`AdminAreaStatsRepository::get_area_stats`] and returns an
//! [`AdminAreaStats`] aggregate. Called by `GET /api/v1/area-stats`.

use std::sync::Arc;

use crate::domain::entity::AdminAreaStats;
use crate::domain::error::DomainError;
use crate::domain::repository::AdminAreaStatsRepository;
use crate::domain::value_object::AreaCode;

/// Usecase for `GET /api/v1/area-stats`.
pub(crate) struct GetAreaStatsUsecase {
    repo: Arc<dyn AdminAreaStatsRepository>,
}

impl GetAreaStatsUsecase {
    /// Construct the usecase with the given repository.
    pub(crate) fn new(repo: Arc<dyn AdminAreaStatsRepository>) -> Self {
        Self { repo }
    }

    /// Fetch aggregated statistics for the given administrative area code.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    #[tracing::instrument(skip(self), fields(usecase = "get_area_stats"))]
    pub(crate) async fn execute(&self, code: &AreaCode) -> Result<AdminAreaStats, DomainError> {
        self.repo.get_area_stats(code).await.inspect(|stats| {
            tracing::debug!(
                code = stats.code.as_str(),
                land_price_count = stats.land_price.count,
                "area-stats query complete"
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{AreaName, FacilityStats, LandPriceStats, RiskStats};
    use crate::domain::repository::mock::MockAdminAreaStatsRepository;

    fn sample_stats() -> AdminAreaStats {
        AdminAreaStats {
            code: AreaCode::parse("13").unwrap(),
            name: AreaName::parse("Tokyo").unwrap(),
            level: "prefecture".into(),
            land_price: LandPriceStats {
                avg_per_sqm: Some(1000.0),
                median_per_sqm: Some(900.0),
                min_per_sqm: Some(500),
                max_per_sqm: Some(2000),
                count: 42,
            },
            risk: RiskStats {
                flood_area_ratio: 0.1,
                steep_slope_area_ratio: 0.05,
                composite_risk: 0.08,
            },
            facilities: FacilityStats {
                schools: 10,
                medical: 5,
            },
        }
    }

    #[tokio::test]
    async fn execute_happy_path_returns_stats() {
        let repo =
            Arc::new(MockAdminAreaStatsRepository::new().with_get_area_stats(Ok(sample_stats())));
        let usecase = GetAreaStatsUsecase::new(repo);
        let code = AreaCode::parse("13").unwrap();

        let result = usecase.execute(&code).await.unwrap();

        assert_eq!(result.code.as_str(), "13");
        assert_eq!(result.land_price.count, 42);
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let repo = Arc::new(
            MockAdminAreaStatsRepository::new()
                .with_get_area_stats(Err(DomainError::Database("boom".into()))),
        );
        let usecase = GetAreaStatsUsecase::new(repo);
        let code = AreaCode::parse("13").unwrap();

        let err = usecase.execute(&code).await.unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

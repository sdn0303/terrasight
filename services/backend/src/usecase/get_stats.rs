//! Usecase: aggregate statistics for a bounding box.
//!
//! Issues all four stats queries (land price, risk, facilities, zoning
//! distribution) in parallel via `tokio::try_join!` and assembles the result
//! into an [`AreaStats`] struct. Called by `GET /api/v1/stats`.

use std::sync::Arc;

use crate::domain::entity::AreaStats;
use crate::domain::error::DomainError;
use crate::domain::repository::StatsRepository;
use crate::domain::value_object::{BBox, PrefCode};

/// Usecase for `GET /api/v1/stats`.
pub(crate) struct GetStatsUsecase {
    stats_repo: Arc<dyn StatsRepository>,
}

impl GetStatsUsecase {
    /// Construct the usecase with the given stats repository.
    pub(crate) fn new(stats_repo: Arc<dyn StatsRepository>) -> Self {
        Self { stats_repo }
    }

    /// Aggregate land-price stats, risk ratios, facility counts, and zoning distribution.
    ///
    /// All four queries run concurrently via `tokio::try_join!`. If any query
    /// fails the entire call fails immediately (fail-fast semantics).
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from any of the four repository calls.
    #[tracing::instrument(skip(self), fields(usecase = "get_stats"))]
    pub(crate) async fn execute(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<AreaStats, DomainError> {
        tokio::try_join!(
            self.stats_repo.calc_land_price_stats(bbox, pref_code),
            self.stats_repo.calc_risk_stats(bbox, pref_code),
            self.stats_repo.count_facilities(bbox, pref_code),
            self.stats_repo.calc_zoning_distribution(bbox, pref_code),
        )
        .map(
            |(land_price, risk, facilities, zoning_distribution)| AreaStats {
                land_price,
                risk,
                facilities,
                zoning_distribution,
            },
        )
        .inspect(|stats| {
            tracing::debug!(
                land_price_count = stats.land_price.count,
                schools = stats.facilities.schools,
                medical = stats.facilities.medical,
                zoning_types = stats.zoning_distribution.len(),
                "stats queries complete"
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::domain::entity::{FacilityStats, LandPriceStats, RiskStats};
    use crate::domain::repository::mock::MockStatsRepository;

    fn sample_bbox() -> BBox {
        BBox::new(35.65, 139.70, 35.70, 139.80).unwrap()
    }

    fn sample_land_price() -> LandPriceStats {
        LandPriceStats {
            avg_per_sqm: Some(1000.0),
            median_per_sqm: Some(900.0),
            min_per_sqm: Some(500),
            max_per_sqm: Some(2000),
            count: 10,
        }
    }

    fn sample_risk() -> RiskStats {
        RiskStats {
            flood_area_ratio: 0.1,
            steep_slope_area_ratio: 0.05,
            composite_risk: 0.08,
        }
    }

    fn sample_facilities() -> FacilityStats {
        FacilityStats {
            schools: 10,
            medical: 5,
        }
    }

    fn sample_zoning() -> HashMap<String, f64> {
        let mut m = HashMap::new();
        m.insert("residential".into(), 0.6);
        m.insert("commercial".into(), 0.4);
        m
    }

    /// Verifies that `execute` exercises all four parallel `tokio::try_join!`
    /// branches and merges their results into a single `AreaStats`.
    #[tokio::test]
    async fn execute_happy_path_joins_all_four_queries() {
        let repo = Arc::new(
            MockStatsRepository::new()
                .with_land_price(Ok(sample_land_price()))
                .with_risk(Ok(sample_risk()))
                .with_facilities(Ok(sample_facilities()))
                .with_zoning_distribution(Ok(sample_zoning())),
        );
        let usecase = GetStatsUsecase::new(repo);

        let result = usecase.execute(&sample_bbox(), None).await.unwrap();

        assert_eq!(result.land_price.count, 10);
        assert_eq!(result.facilities.schools, 10);
        assert_eq!(result.zoning_distribution.len(), 2);
    }

    #[tokio::test]
    async fn execute_propagates_db_error_from_any_branch() {
        // Queue success for three branches; land_price fails.
        let repo = Arc::new(
            MockStatsRepository::new()
                .with_land_price(Err(DomainError::Database("boom".into())))
                .with_risk(Ok(sample_risk()))
                .with_facilities(Ok(sample_facilities()))
                .with_zoning_distribution(Ok(sample_zoning())),
        );
        let usecase = GetStatsUsecase::new(repo);

        let err = usecase.execute(&sample_bbox(), None).await.unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

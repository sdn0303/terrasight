use std::sync::Arc;

use crate::domain::entity::AreaStats;
use crate::domain::error::DomainError;
use crate::domain::repository::StatsRepository;
use crate::domain::value_object::BBox;

pub struct GetStatsUsecase {
    stats_repo: Arc<dyn StatsRepository>,
}

impl GetStatsUsecase {
    pub fn new(stats_repo: Arc<dyn StatsRepository>) -> Self {
        Self { stats_repo }
    }

    /// Aggregate area statistics for the given bounding box.
    ///
    /// All 4 stats queries execute in parallel.
    pub async fn execute(&self, bbox: &BBox) -> Result<AreaStats, DomainError> {
        let (land_price, risk, facilities, zoning_distribution) = tokio::try_join!(
            self.stats_repo.calc_land_price_stats(bbox),
            self.stats_repo.calc_risk_stats(bbox),
            self.stats_repo.count_facilities(bbox),
            self.stats_repo.calc_zoning_distribution(bbox),
        )?;

        tracing::debug!(
            land_price_count = land_price.count,
            schools = facilities.schools,
            medical = facilities.medical,
            zoning_types = zoning_distribution.len(),
            "stats queries complete"
        );

        Ok(AreaStats {
            land_price,
            risk,
            facilities,
            zoning_distribution,
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

        let result = usecase.execute(&sample_bbox()).await.unwrap();

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

        let err = usecase.execute(&sample_bbox()).await.unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

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

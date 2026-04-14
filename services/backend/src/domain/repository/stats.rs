//! [`StatsRepository`] trait — aggregate area statistics over a spatial bounding box.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::model::{BBox, FacilityStats, LandPriceStats, PrefCode, RiskStats};

/// Repository for aggregate area statistics.
///
/// Each method runs an aggregating SQL query over a spatial bounding box and
/// returns a summary value. Called concurrently by the stats usecase via
/// `tokio::join!`.
///
/// Implemented by `PgStatsRepository` in the `infra` layer.
#[async_trait]
pub trait StatsRepository: Send + Sync {
    /// Compute land price statistics (min, max, avg, median) within the bbox.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn calc_land_price_stats(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<LandPriceStats, DomainError>;

    /// Compute flood and steep-slope risk statistics within the bbox.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn calc_risk_stats(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<RiskStats, DomainError>;

    /// Count school and medical facilities within the bbox.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn count_facilities(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<FacilityStats, DomainError>;

    /// Compute the share of each zoning type within the bbox.
    ///
    /// Returns a map from zone code string to area fraction (values sum to 1.0).
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn calc_zoning_distribution(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<HashMap<String, f64>, DomainError>;
}

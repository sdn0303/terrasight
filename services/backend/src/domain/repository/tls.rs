//! [`TlsRepository`] trait — Total Location Score sub-score data queries.

use async_trait::async_trait;

use crate::domain::entity::{MedicalStats, PriceRecord, SchoolStats, ZScoreResult};
use crate::domain::error::DomainError;
use crate::domain::value_object::Coord;

/// Repository for the Total Location Score (TLS) sub-score data queries.
///
/// Each method fetches the data needed for one TLS sub-score component.
/// The opportunities usecase fans out over all methods concurrently via
/// `tokio::join!` for each candidate record.
///
/// Implemented by `PgTlsRepository` in the `infra` layer.
#[async_trait]
pub trait TlsRepository: Send + Sync {
    /// Multi-year land prices near the given coordinate (nearest address within 1km).
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_nearest_prices(&self, coord: &Coord) -> Result<Vec<PriceRecord>, DomainError>;

    /// Maximum flood depth rank within 500m buffer. `None` = outside any flood zone.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_flood_depth_rank(&self, coord: &Coord) -> Result<Option<i32>, DomainError>;

    /// Whether steep slope hazard exists within 500m.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn has_steep_slope_nearby(&self, coord: &Coord) -> Result<bool, DomainError>;

    /// School statistics within 800m: count, primary presence, junior-high presence.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_schools_nearby(&self, coord: &Coord) -> Result<SchoolStats, DomainError>;

    /// Medical facility statistics within 1000m: hospital/clinic counts and total beds.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_medical_nearby(&self, coord: &Coord) -> Result<MedicalStats, DomainError>;

    /// Floor area ratio at the given point from the containing zoning polygon.
    ///
    /// Returns `None` if the point is not within any zoning polygon.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_zoning_far(&self, coord: &Coord) -> Result<Option<f64>, DomainError>;

    /// Z-score of the point's land price relative to all prices in the same zoning type.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn calc_price_z_score(&self, coord: &Coord) -> Result<ZScoreResult, DomainError>;

    /// Count of land price records within 500m from the latest available year.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn count_recent_transactions(&self, coord: &Coord) -> Result<i64, DomainError>;
}

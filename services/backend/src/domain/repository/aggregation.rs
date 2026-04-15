//! [`AggregationRepository`] trait — polygon aggregation queries for choropleth map layers.

use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::model::{BBox, LandPriceAggRow, PrefCode, TransactionAggRow};

/// Repository for polygon-level aggregation queries (choropleth layers).
///
/// Joins `admin_boundaries` with domain-specific tables and returns typed
/// aggregation rows. GeoJSON assembly is the usecase layer's responsibility.
///
/// Implemented by `PgAggregationRepository` in the `infra` layer.
#[async_trait]
pub trait AggregationRepository: Send + Sync {
    /// Land price aggregation per municipality polygon.
    ///
    /// Joins `admin_boundaries` (municipality level) with `land_prices` for
    /// the two most recent survey years. Returns one [`LandPriceAggRow`] per
    /// municipality that has at least one land price record.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure or
    /// [`DomainError::Timeout`] when the query exceeds the configured deadline.
    async fn land_price_aggregation(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<Vec<LandPriceAggRow>, DomainError>;

    /// Transaction aggregation per municipality polygon.
    ///
    /// Joins `admin_boundaries` with `transaction_prices` via
    /// `city_code = admin_code`. Returns one [`TransactionAggRow`] per
    /// municipality that has at least one transaction record.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure or
    /// [`DomainError::Timeout`] when the query exceeds the configured deadline.
    async fn transaction_aggregation(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<Vec<TransactionAggRow>, DomainError>;
}

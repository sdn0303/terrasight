//! [`TrendRepository`] trait — price trend time-series queries for a coordinate.

use async_trait::async_trait;

use crate::domain::entity::{TrendLocation, TrendPoint};
use crate::domain::error::DomainError;
use crate::domain::value_object::{Coord, YearsLookback};

/// Repository for price trend time-series queries.
///
/// Implemented by `PgTrendRepository` in the `infra` layer.
#[async_trait]
pub trait TrendRepository: Send + Sync {
    /// Fetch price trend data for the nearest land price observation point.
    ///
    /// Searches within a 2 km radius of `coord`. Returns `None` when no
    /// observation point exists within that radius.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_trend(
        &self,
        coord: Coord,
        years: YearsLookback,
    ) -> Result<Option<(TrendLocation, Vec<TrendPoint>)>, DomainError>;
}

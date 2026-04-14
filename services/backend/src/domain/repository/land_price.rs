//! [`LandPriceRepository`] trait — land price spatial queries for the map view and opportunities pipeline.

use async_trait::async_trait;

use crate::domain::entity::{LayerResult, OpportunityRecord, PricePerSqm, ZoneCode};
use crate::domain::error::DomainError;
use crate::domain::value_object::{BBox, PrefCode, Year, ZoomLevel};

/// Repository for land price spatial queries (dedicated `/api/v1/landprice` endpoint).
///
/// Provides year-filtered and year-range GeoJSON queries for the map view as
/// well as raw record fetching for the opportunities pipeline.
///
/// Implemented by `PgLandPriceRepository` in the `infra` layer.
#[async_trait]
pub trait LandPriceRepository: Send + Sync {
    /// Fetch land price GeoJSON features filtered by year, bounding box, and zoom.
    ///
    /// The `zoom` level is used to compute a dynamic feature limit via
    /// `compute_feature_limit`. Returns [`LayerResult`] with truncation metadata.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_by_year_and_bbox(
        &self,
        year: Year,
        bbox: &BBox,
        zoom: ZoomLevel,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError>;

    /// Fetch land price GeoJSON features across a year range for time-machine animation.
    ///
    /// Returns all survey years within `[from_year, to_year]` in a single
    /// [`LayerResult`]. The feature count cap applies across all years combined.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_all_years_by_bbox(
        &self,
        from_year: Year,
        to_year: Year,
        bbox: &BBox,
        zoom: ZoomLevel,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError>;

    /// Fetch raw land price records for the `/api/v1/opportunities` pipeline.
    ///
    /// Returns up to `limit` [`OpportunityRecord`] rows ordered by the
    /// database default (primary key). The usecase runs TLS enrichment on
    /// the returned pool before applying user-facing filters and pagination.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_for_opportunities(
        &self,
        bbox: &BBox,
        limit: u32,
        price_range: Option<(PricePerSqm, PricePerSqm)>,
        zones: &[ZoneCode],
        pref_code: Option<&PrefCode>,
    ) -> Result<Vec<OpportunityRecord>, DomainError>;
}

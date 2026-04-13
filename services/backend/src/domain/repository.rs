//! Repository trait contracts for domain data access.
//!
//! Each trait in this module defines the interface between the `usecase` layer
//! and the `infra` layer. Usecases depend on these traits via `Arc<dyn Trait>`;
//! the `infra` layer provides concrete PostgreSQL implementations
//! (`PgLayerRepository`, `PgStatsRepository`, etc.).
//!
//! ## Design principles
//!
//! - Traits are scoped to a single aggregate root (one table family per trait).
//! - All methods are `async` (via `async_trait`) and return `Result<_, DomainError>`.
//! - The infra layer must never let `sqlx::Error` or other framework errors
//!   escape — they are converted to [`DomainError::Database`] at the boundary.
//! - `mock` (test-only) provides in-process test doubles for every trait in
//!   this module, gated behind `#[cfg(test)]`.

use std::collections::HashMap;

use async_trait::async_trait;

use crate::domain::appraisal::AppraisalDetail;
use crate::domain::entity::{
    AdminAreaStats, FacilityStats, LandPriceStats, LayerResult, MedicalStats, OpportunityRecord,
    PricePerSqm, PriceRecord, RiskStats, SchoolStats, TrendLocation, TrendPoint, ZScoreResult,
    ZoneCode,
};
use crate::domain::error::DomainError;
use crate::domain::municipality::Municipality;
use crate::domain::transaction::{TransactionDetail, TransactionSummary};
use crate::domain::value_object::{
    AreaCode, BBox, Coord, LayerType, PrefCode, Year, YearsLookback, ZoomLevel,
};

/// Repository for map layer GeoJSON features.
///
/// Uses a single enum-dispatched entry point ([`find_layer`]) so the usecase
/// can fan out over all [`LayerType`] variants concurrently without the trait
/// growing a new method each time a layer is added.
///
/// Implemented by `PgLayerRepository` in the `infra` layer.
///
/// [`find_layer`]: LayerRepository::find_layer
#[async_trait]
pub trait LayerRepository: Send + Sync {
    /// Fetch GeoJSON features for a single map layer within the given bbox.
    ///
    /// The feature count is capped by a zoom-dependent limit; [`LayerResult`]
    /// carries a `truncated` flag when the cap was hit.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_layer(
        &self,
        layer: LayerType,
        bbox: &BBox,
        zoom: ZoomLevel,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError>;
}

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

/// Repository for administrative area aggregate statistics.
///
/// Accepts both prefecture (2-digit) and municipality (5-digit) area codes.
///
/// Implemented by `PgAdminAreaStatsRepository` in the `infra` layer.
#[async_trait]
pub trait AdminAreaStatsRepository: Send + Sync {
    /// Fetch aggregated statistics for the given administrative area code.
    ///
    /// `code` is a prefecture code (e.g. `"13"`) or municipality code (e.g. `"13105"`).
    async fn get_area_stats(&self, code: &AreaCode) -> Result<AdminAreaStats, DomainError>;
}

/// Repository for database health probes.
///
/// Implemented by `PgHealthRepository` in the `infra` layer.
#[async_trait]
pub trait HealthRepository: Send + Sync {
    /// Check database connectivity (SELECT 1).
    async fn check_connection(&self) -> bool;
}

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
    async fn find_nearest_prices(&self, coord: &Coord) -> Result<Vec<PriceRecord>, DomainError>;

    /// Maximum flood depth rank within 500m buffer. `None` = outside any flood zone.
    async fn find_flood_depth_rank(&self, coord: &Coord) -> Result<Option<i32>, DomainError>;

    /// Whether steep slope hazard exists within 500m.
    async fn has_steep_slope_nearby(&self, coord: &Coord) -> Result<bool, DomainError>;

    /// School statistics within 800m: count, primary presence, junior-high presence.
    async fn find_schools_nearby(&self, coord: &Coord) -> Result<SchoolStats, DomainError>;

    /// Medical facility statistics within 1000m: hospital/clinic counts and total beds.
    async fn find_medical_nearby(&self, coord: &Coord) -> Result<MedicalStats, DomainError>;

    /// Floor area ratio at the given point from the containing zoning polygon.
    /// `None` if the point is not within any zoning polygon.
    async fn find_zoning_far(&self, coord: &Coord) -> Result<Option<f64>, DomainError>;

    /// Z-score of the point's land price relative to all prices in the same zoning type.
    async fn calc_price_z_score(&self, coord: &Coord) -> Result<ZScoreResult, DomainError>;

    /// Count of land price records within 500m from the latest available year.
    async fn count_recent_transactions(&self, coord: &Coord) -> Result<i64, DomainError>;
}

/// Repository for real-estate transaction data.
///
/// Queries the `transactions` table sourced from MLIT reinfolib XPT003.
///
/// Implemented by `PgTransactionRepository` in the `infra` layer.
#[async_trait]
pub trait TransactionRepository: Send + Sync {
    /// Fetch aggregated transaction summaries per city/year/property_type.
    ///
    /// `year_from` optionally restricts results to records on or after that year.
    /// `property_type` optionally restricts to a single property type string.
    async fn find_transaction_summary(
        &self,
        pref_code: &PrefCode,
        year_from: Option<&Year>,
        property_type: Option<&str>,
    ) -> Result<Vec<TransactionSummary>, DomainError>;

    /// Fetch individual transaction records for a given city code.
    ///
    /// `city_code` is a raw 5-digit JIS X 0402 string. CityCode は導入済みだが、
    /// handler 層でバリデーション済みのため trait は &str を維持。
    async fn find_transactions(
        &self,
        city_code: &str,
        year_from: Option<&Year>,
        limit: u32,
    ) -> Result<Vec<TransactionDetail>, DomainError>;
}

/// Repository for official land appraisal (鑑定評価) records.
///
/// Queries the `appraisals` table sourced from MLIT reinfolib.
///
/// Implemented by `PgAppraisalRepository` in the `infra` layer.
#[async_trait]
pub trait AppraisalRepository: Send + Sync {
    /// Fetch appraisal records for a prefecture, optionally filtered by city code.
    ///
    /// `city_code` is a raw 5-digit JIS X 0402 string. CityCode は導入済みだが、
    /// handler 層でバリデーション済みのため trait は &str を維持。
    async fn find_appraisals(
        &self,
        pref_code: &PrefCode,
        city_code: Option<&str>,
    ) -> Result<Vec<AppraisalDetail>, DomainError>;
}

/// Repository for municipality lookup data.
///
/// Provides the list of [`Municipality`] records for a given prefecture,
/// used by the `/api/v1/municipalities` endpoint.
///
/// Implemented by `PgMunicipalityRepository` in the `infra` layer.
#[async_trait]
pub trait MunicipalityRepository: Send + Sync {
    /// Fetch all municipalities for the given prefecture.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_municipalities(
        &self,
        pref_code: &PrefCode,
    ) -> Result<Vec<Municipality>, DomainError>;
}

#[cfg(test)]
pub mod mock;

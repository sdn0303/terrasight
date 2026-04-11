use std::collections::HashMap;

use async_trait::async_trait;

use crate::domain::entity::{LayerResult, MedicalStats, SchoolStats, ZScoreResult, *};
use crate::domain::error::DomainError;
use crate::domain::value_object::*;

// ─── Layer Data ──────────────────────────────────────
// Enum-dispatched: a single `find_layer` entry point replaces the
// previous six per-layer methods, enabling the caller (usecase layer)
// to drive concurrent fan-out via `LayerType` without the trait growing
// a new method each time a layer is added.

#[async_trait]
pub trait LayerRepository: Send + Sync {
    async fn find_layer(
        &self,
        layer: LayerType,
        bbox: &BBox,
        zoom: ZoomLevel,
    ) -> Result<LayerResult, DomainError>;
}

// ─── Stats ───────────────────────────────────────────

#[async_trait]
pub trait StatsRepository: Send + Sync {
    async fn calc_land_price_stats(&self, bbox: &BBox) -> Result<LandPriceStats, DomainError>;
    async fn calc_risk_stats(&self, bbox: &BBox) -> Result<RiskStats, DomainError>;
    async fn count_facilities(&self, bbox: &BBox) -> Result<FacilityStats, DomainError>;
    async fn calc_zoning_distribution(
        &self,
        bbox: &BBox,
    ) -> Result<HashMap<String, f64>, DomainError>;
}

// ─── Trend ───────────────────────────────────────────

#[async_trait]
pub trait TrendRepository: Send + Sync {
    /// Price trend data for the nearest observation point within 2km.
    async fn find_trend(
        &self,
        coord: Coord,
        years: YearsLookback,
    ) -> Result<Option<(TrendLocation, Vec<TrendPoint>)>, DomainError>;
}

// ─── Land Prices (dedicated v1 endpoint) ─────────────

#[async_trait]
pub trait LandPriceRepository: Send + Sync {
    /// Fetch land price GeoJSON features filtered by year, bounding box, and zoom.
    ///
    /// The `zoom` level is used to compute a dynamic feature limit via
    /// `compute_feature_limit`. Returns [`LayerResult`] with truncation metadata.
    async fn find_by_year_and_bbox(
        &self,
        year: Year,
        bbox: &BBox,
        zoom: ZoomLevel,
    ) -> Result<LayerResult, DomainError>;

    /// Fetch land price GeoJSON features across a year range for time machine animation.
    ///
    /// Returns features with `year` property included so the client can filter
    /// client-side via MapLibre `setFilter`. The feature limit is multiplied by the
    /// number of years to accommodate multi-year data in a single response.
    async fn find_all_years_by_bbox(
        &self,
        from_year: Year,
        to_year: Year,
        bbox: &BBox,
        zoom: ZoomLevel,
    ) -> Result<LayerResult, DomainError>;

    /// Fetch enriched land price records for the `/api/v1/opportunities` endpoint.
    ///
    /// Returns raw [`OpportunityRecord`]s (pre-TLS-enrichment) filtered by `bbox`
    /// and an optional price range and zone whitelist. The caller must apply TLS
    /// scoring, risk classification, and signal derivation on top.
    ///
    /// Uses a spatial join with the `zoning` table to populate
    /// [`BuildingCoverageRatio`] and [`FloorAreaRatio`]. Results are ordered by
    /// `price_per_sqm DESC` and paginated via `limit`/`offset`.
    async fn find_for_opportunities(
        &self,
        bbox: &BBox,
        limit: OpportunityLimit,
        offset: OpportunityOffset,
        price_range: Option<(PricePerSqm, PricePerSqm)>,
        zones: &[ZoneCode],
    ) -> Result<Vec<OpportunityRecord>, DomainError>;
}

// ─── Admin Area Stats ────────────────────────────────

#[async_trait]
pub trait AdminAreaStatsRepository: Send + Sync {
    /// Fetch aggregated statistics for the given administrative area code.
    ///
    /// `code` is a prefecture code (e.g. `"13"`) or municipality code (e.g. `"13105"`).
    async fn get_area_stats(&self, code: &AreaCode) -> Result<AdminAreaStats, DomainError>;
}

// ─── Health ──────────────────────────────────────────

#[async_trait]
pub trait HealthRepository: Send + Sync {
    /// Check database connectivity (SELECT 1).
    async fn check_connection(&self) -> bool;
}

// ─── TLS (Total Location Score) ──────────────────────

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

#[cfg(test)]
pub mod mock;

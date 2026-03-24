use std::collections::HashMap;

use async_trait::async_trait;

use crate::domain::entity::{LayerResult, *};
use crate::domain::error::DomainError;
use crate::domain::value_object::*;

// ─── Area Data ───────────────────────────────────────
// Per-layer methods: each table has different schema, and future
// layer-specific return types (e.g., `LandPriceFeature`) are possible.

#[async_trait]
pub trait AreaRepository: Send + Sync {
    async fn find_land_prices(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError>;
    async fn find_zoning(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError>;
    async fn find_flood_risk(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError>;
    async fn find_steep_slope(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError>;
    async fn find_schools(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError>;
    async fn find_medical(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError>;
}

// ─── Score ───────────────────────────────────────────

#[async_trait]
pub trait ScoreRepository: Send + Sync {
    /// Multi-year prices near the given coordinate (nearest address within 1km).
    async fn find_nearest_prices(&self, coord: &Coord) -> Result<Vec<PriceRecord>, DomainError>;

    /// Flood risk overlap ratio (0.0–1.0) within 500m buffer.
    async fn calc_flood_overlap(&self, coord: &Coord) -> Result<f64, DomainError>;

    /// Whether steep slope hazard exists within 500m.
    async fn has_steep_slope_nearby(&self, coord: &Coord) -> Result<bool, DomainError>;

    /// School count + nearest distance (m) within 1km.
    async fn count_schools_nearby(&self, coord: &Coord) -> Result<(i64, f64), DomainError>;

    /// Medical facility count + nearest distance (m) within 1km.
    async fn count_medical_nearby(&self, coord: &Coord) -> Result<(i64, f64), DomainError>;
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
        coord: &Coord,
        years: i32,
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
        year: &Year,
        bbox: &BBox,
        zoom: u32,
    ) -> Result<LayerResult, DomainError>;
}

// ─── Health ──────────────────────────────────────────

#[async_trait]
pub trait HealthRepository: Send + Sync {
    /// Check database connectivity (SELECT 1).
    async fn check_connection(&self) -> bool;
}

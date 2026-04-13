//! Shared domain types used by both Backend and WASM.

use serde::Serialize;

/// Aggregated land price statistics within a bounding box.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LandPriceStats {
    pub avg_per_sqm: Option<f64>,
    pub median_per_sqm: Option<f64>,
    pub min_per_sqm: Option<i64>,
    pub max_per_sqm: Option<i64>,
    pub count: i64,
}

/// Risk statistics within a bounding box.
#[derive(Debug, Clone, Serialize)]
pub struct RiskStats {
    pub flood_area_ratio: f64,
    pub steep_slope_area_ratio: f64,
    pub composite_risk: f64,
}

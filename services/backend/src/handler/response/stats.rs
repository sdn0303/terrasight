//! Response DTOs for `GET /api/v1/stats`.

use serde::Serialize;

use crate::domain::entity::AreaStats;

/// Top-level response for `GET /api/v1/stats`.
///
/// Aggregates computed over the bounding box supplied in the request.
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    /// Land price summary statistics.
    pub land_price: LandPriceStatsDto,
    /// Disaster risk metrics.
    pub risk: RiskStatsDto,
    /// Public facility counts.
    pub facilities: FacilityStatsDto,
    /// Map from zoning type name (Japanese) to its fractional share of the
    /// total zoning area within the bounding box (values sum to ≤ 1.0).
    pub zoning_distribution: std::collections::HashMap<String, f64>,
}

/// Land price statistics nested inside [`StatsResponse`].
#[derive(Debug, Serialize)]
pub struct LandPriceStatsDto {
    /// Mean land price per square metre in JPY. `null` when no data.
    pub avg_per_sqm: Option<f64>,
    /// Median land price per square metre in JPY. `null` when no data.
    pub median_per_sqm: Option<f64>,
    /// Minimum observed land price per square metre in JPY. `null` when no data.
    pub min_per_sqm: Option<i64>,
    /// Maximum observed land price per square metre in JPY. `null` when no data.
    pub max_per_sqm: Option<i64>,
    /// Number of land price survey points within the bounding box.
    pub count: i64,
}

/// Disaster risk metrics nested inside [`StatsResponse`].
#[derive(Debug, Serialize)]
pub struct RiskStatsDto {
    /// Fraction of the bounding box covered by flood-risk zones (0.0 – 1.0).
    pub flood_area_ratio: f64,
    /// Fraction covered by steep-slope hazard zones (0.0 – 1.0).
    pub steep_slope_area_ratio: f64,
    /// Composite disaster risk score (0.0 – 1.0, higher = riskier).
    pub composite_risk: f64,
}

/// Facility counts nested inside [`StatsResponse`].
#[derive(Debug, Serialize)]
pub struct FacilityStatsDto {
    /// Number of schools (primary + secondary) within the bounding box.
    pub schools: i64,
    /// Number of medical facilities (hospitals + clinics) within the bounding box.
    pub medical: i64,
}

impl From<AreaStats> for StatsResponse {
    fn from(s: AreaStats) -> Self {
        Self {
            land_price: LandPriceStatsDto {
                avg_per_sqm: s.land_price.avg_per_sqm,
                median_per_sqm: s.land_price.median_per_sqm,
                min_per_sqm: s.land_price.min_per_sqm,
                max_per_sqm: s.land_price.max_per_sqm,
                count: s.land_price.count,
            },
            risk: RiskStatsDto {
                flood_area_ratio: s.risk.flood_area_ratio,
                steep_slope_area_ratio: s.risk.steep_slope_area_ratio,
                composite_risk: s.risk.composite_risk,
            },
            facilities: FacilityStatsDto {
                schools: s.facilities.schools,
                medical: s.facilities.medical,
            },
            zoning_distribution: s.zoning_distribution,
        }
    }
}

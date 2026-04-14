//! Response DTOs for `GET /api/v1/area-stats`.
//!
//! These types were previously inlined in `handler/area_stats.rs`; they
//! are hoisted here so the response layer owns every handler DTO and
//! the handler file can focus on request wiring.

use serde::Serialize;

use crate::domain::entity::AdminAreaStats;

/// Top-level response for `GET /api/v1/area-stats`.
#[derive(Debug, Serialize)]
pub struct AreaStatsResponse {
    /// Administrative area code (2-digit prefecture or 5-digit municipality).
    pub code: String,
    /// Human-readable area name in Japanese (e.g. `"東京都"` or `"新宿区"`).
    pub name: String,
    /// Granularity of the area: `"prefecture"` or `"municipality"`.
    pub level: String,
    /// Land price aggregates for the area.
    pub land_price: AreaLandPriceDto,
    /// Disaster risk metrics for the area.
    pub risk: AreaRiskDto,
    /// Public facility counts for the area.
    pub facilities: AreaFacilitiesDto,
}

/// Land price aggregates nested inside [`AreaStatsResponse`].
#[derive(Debug, Serialize)]
pub struct AreaLandPriceDto {
    /// Mean land price per square metre in JPY. `null` when no data.
    pub avg_per_sqm: Option<f64>,
    /// Median land price per square metre in JPY. `null` when no data.
    pub median_per_sqm: Option<f64>,
    /// Number of land price survey points in this area.
    pub count: i64,
}

/// Disaster risk metrics nested inside [`AreaStatsResponse`].
#[derive(Debug, Serialize)]
pub struct AreaRiskDto {
    /// Fraction of the area covered by flood-risk zones (0.0 – 1.0).
    pub flood_area_ratio: f64,
    /// Composite disaster risk score (0.0 – 1.0, higher = riskier).
    pub composite_risk: f64,
}

/// Public facility counts nested inside [`AreaStatsResponse`].
#[derive(Debug, Serialize)]
pub struct AreaFacilitiesDto {
    /// Number of schools (primary + secondary) within the area.
    pub schools: i64,
    /// Number of medical facilities (hospitals + clinics) within the area.
    pub medical: i64,
}

impl From<AdminAreaStats> for AreaStatsResponse {
    fn from(stats: AdminAreaStats) -> Self {
        Self {
            code: stats.code.as_str().to_owned(),
            name: stats.name.as_str().to_owned(),
            level: stats.level,
            land_price: AreaLandPriceDto {
                avg_per_sqm: stats.land_price.avg_per_sqm,
                median_per_sqm: stats.land_price.median_per_sqm,
                count: stats.land_price.count,
            },
            risk: AreaRiskDto {
                flood_area_ratio: stats.risk.flood_area_ratio,
                composite_risk: stats.risk.composite_risk,
            },
            facilities: AreaFacilitiesDto {
                schools: stats.facilities.schools,
                medical: stats.facilities.medical,
            },
        }
    }
}

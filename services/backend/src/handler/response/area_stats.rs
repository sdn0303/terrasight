//! Response DTOs for `GET /api/area-stats`.
//!
//! These types were previously inlined in `handler/area_stats.rs`; they
//! are hoisted here so the response layer owns every handler DTO and
//! the handler file can focus on request wiring.

use serde::Serialize;

use crate::domain::entity::AdminAreaStats;

/// Response for `GET /api/area-stats`.
#[derive(Debug, Serialize)]
pub struct AreaStatsResponse {
    pub code: String,
    pub name: String,
    pub level: String,
    pub land_price: AreaLandPriceDto,
    pub risk: AreaRiskDto,
    pub facilities: AreaFacilitiesDto,
}

#[derive(Debug, Serialize)]
pub struct AreaLandPriceDto {
    pub avg_per_sqm: Option<f64>,
    pub median_per_sqm: Option<f64>,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct AreaRiskDto {
    pub flood_area_ratio: f64,
    pub composite_risk: f64,
}

#[derive(Debug, Serialize)]
pub struct AreaFacilitiesDto {
    pub schools: i64,
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

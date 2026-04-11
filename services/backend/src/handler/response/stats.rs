//! Response DTOs for `GET /api/stats`.

use serde::Serialize;

use crate::domain::entity::AreaStats;

/// Response for `GET /api/stats`.
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub land_price: LandPriceStatsDto,
    pub risk: RiskStatsDto,
    pub facilities: FacilityStatsDto,
    pub zoning_distribution: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Serialize)]
pub struct LandPriceStatsDto {
    pub avg_per_sqm: Option<f64>,
    pub median_per_sqm: Option<f64>,
    pub min_per_sqm: Option<i64>,
    pub max_per_sqm: Option<i64>,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct RiskStatsDto {
    pub flood_area_ratio: f64,
    pub steep_slope_area_ratio: f64,
    pub composite_risk: f64,
}

#[derive(Debug, Serialize)]
pub struct FacilityStatsDto {
    pub schools: i64,
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

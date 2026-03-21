use serde::Serialize;

use crate::domain::constants::SCORE_DISCLAIMER;
use crate::domain::entity::*;
use crate::domain::value_object::*;

pub use realestate_api_core::response::{FeatureCollectionDto, FeatureDto};

/// Response for `GET /api/health`.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub db_connected: bool,
    pub reinfolib_key_set: bool,
    pub version: &'static str,
}

impl From<HealthStatus> for HealthResponse {
    fn from(h: HealthStatus) -> Self {
        Self {
            status: h.status,
            db_connected: h.db_connected,
            reinfolib_key_set: h.reinfolib_key_set,
            version: h.version,
        }
    }
}

/// Response for `GET /api/score`.
#[derive(Debug, Serialize)]
pub struct ScoreResponse {
    pub score: f64,
    pub components: ScoreComponentsDto,
    pub metadata: ScoreMetadataDto,
}

#[derive(Debug, Serialize)]
pub struct ScoreComponentsDto {
    pub trend: ScoreDetailDto,
    pub risk: ScoreDetailDto,
    pub access: ScoreDetailDto,
    pub yield_potential: ScoreDetailDto,
}

#[derive(Debug, Serialize)]
pub struct ScoreDetailDto {
    pub value: f64,
    pub max: f64,
    pub detail: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ScoreMetadataDto {
    pub calculated_at: String,
    pub data_freshness: String,
    pub disclaimer: String,
}

impl From<InvestmentScore> for ScoreResponse {
    fn from(s: InvestmentScore) -> Self {
        Self {
            score: s.total(),
            components: ScoreComponentsDto {
                trend: ScoreDetailDto {
                    value: s.trend.value,
                    max: s.trend.max,
                    detail: s.trend.detail,
                },
                risk: ScoreDetailDto {
                    value: s.risk.value,
                    max: s.risk.max,
                    detail: s.risk.detail,
                },
                access: ScoreDetailDto {
                    value: s.access.value,
                    max: s.access.max,
                    detail: s.access.detail,
                },
                yield_potential: ScoreDetailDto {
                    value: s.yield_potential.value,
                    max: s.yield_potential.max,
                    detail: s.yield_potential.detail,
                },
            },
            metadata: ScoreMetadataDto {
                calculated_at: chrono::Utc::now().to_rfc3339(),
                data_freshness: s.data_freshness,
                disclaimer: SCORE_DISCLAIMER.to_string(),
            },
        }
    }
}

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

/// Response for `GET /api/trend`.
#[derive(Debug, Serialize)]
pub struct TrendResponse {
    pub location: TrendLocationDto,
    pub data: Vec<TrendPointDto>,
    pub cagr: f64,
    pub direction: String,
}

#[derive(Debug, Serialize)]
pub struct TrendLocationDto {
    pub address: String,
    pub distance_m: f64,
}

#[derive(Debug, Serialize)]
pub struct TrendPointDto {
    pub year: i32,
    pub price_per_sqm: i64,
}

impl From<TrendAnalysis> for TrendResponse {
    fn from(t: TrendAnalysis) -> Self {
        Self {
            location: TrendLocationDto {
                address: t.location.address,
                distance_m: t.location.distance_m,
            },
            data: t
                .data
                .into_iter()
                .map(|p| TrendPointDto {
                    year: p.year,
                    price_per_sqm: p.price_per_sqm,
                })
                .collect(),
            cagr: t.cagr,
            direction: t.direction.as_str().to_string(),
        }
    }
}

/// Convert a domain [`GeoFeature`] to a [`FeatureDto`] for JSON serialization.
///
/// Bridges the domain entity to the lib's domain-independent DTO.
pub fn geo_feature_to_dto(f: crate::domain::entity::GeoFeature) -> FeatureDto {
    FeatureDto::new(f.geometry.r#type, f.geometry.coordinates, f.properties)
}

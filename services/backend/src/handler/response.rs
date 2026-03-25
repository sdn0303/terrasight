use realestate_geo_math::spatial::point_to_polygon;
use serde::Serialize;

use crate::domain::constants::SCORE_DISCLAIMER;
use crate::domain::entity::*;
use crate::domain::scoring::tls::CrossAnalysis;
use crate::domain::value_object::*;
use crate::usecase::compute_tls::{AxesOutput, AxisOutput, SubScoreOutput, TlsOutput};

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

/// GeoJSON FeatureCollection response with truncation metadata.
///
/// Returned by area-data and land-prices handlers so that MapLibre GL clients
/// can detect when the result has been capped and prompt the user to zoom in.
#[derive(Debug, Serialize)]
pub struct LayerResponseDto {
    /// Always `"FeatureCollection"`.
    pub r#type: String,
    pub features: Vec<FeatureDto>,
    /// `true` when the repository capped the result at `limit`.
    pub truncated: bool,
    /// Number of features actually returned (after truncation).
    pub count: usize,
    /// The per-layer limit that was applied.
    pub limit: i64,
}

impl LayerResponseDto {
    /// Construct from a feature list and truncation metadata.
    pub fn new(features: Vec<FeatureDto>, truncated: bool, limit: i64) -> Self {
        let count = features.len();
        Self {
            r#type: "FeatureCollection".to_string(),
            features,
            truncated,
            count,
            limit,
        }
    }
}

// ─── TLS Response ─────────────────────────────────────────────────────────────

/// Response for `GET /api/score` (TLS system).
#[derive(Debug, Serialize)]
pub struct TlsResponse {
    pub location: LocationDto,
    pub tls: TlsSummaryDto,
    pub axes: AxesDto,
    pub cross_analysis: CrossAnalysisDto,
    pub metadata: TlsMetadataDto,
}

#[derive(Debug, Serialize)]
pub struct LocationDto {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Serialize)]
pub struct TlsSummaryDto {
    pub score: f64,
    pub grade: &'static str,
    pub label: &'static str,
}

#[derive(Debug, Serialize)]
pub struct AxesDto {
    pub disaster: AxisDto,
    pub terrain: AxisDto,
    pub livability: AxisDto,
    pub future: AxisDto,
    pub price: AxisDto,
}

#[derive(Debug, Serialize)]
pub struct AxisDto {
    pub score: f64,
    pub weight: f64,
    pub confidence: f64,
    pub sub: Vec<SubScoreDto>,
}

#[derive(Debug, Serialize)]
pub struct SubScoreDto {
    pub id: &'static str,
    pub score: f64,
    pub available: bool,
    pub detail: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct CrossAnalysisDto {
    pub value_discovery: f64,
    pub demand_signal: f64,
    pub ground_safety: f64,
}

#[derive(Debug, Serialize)]
pub struct TlsMetadataDto {
    pub calculated_at: String,
    pub weight_preset: String,
    pub data_freshness: String,
    pub disclaimer: String,
}

fn axis_to_dto(axis: AxisOutput) -> AxisDto {
    AxisDto {
        score: axis.score,
        weight: axis.weight,
        confidence: axis.confidence,
        sub: axis.sub_scores.into_iter().map(sub_score_to_dto).collect(),
    }
}

fn sub_score_to_dto(s: SubScoreOutput) -> SubScoreDto {
    SubScoreDto {
        id: s.id,
        score: s.score,
        available: s.available,
        detail: s.detail,
    }
}

fn axes_to_dto(axes: AxesOutput) -> AxesDto {
    AxesDto {
        disaster: axis_to_dto(axes.disaster),
        terrain: axis_to_dto(axes.terrain),
        livability: axis_to_dto(axes.livability),
        future: axis_to_dto(axes.future),
        price: axis_to_dto(axes.price),
    }
}

fn cross_analysis_to_dto(ca: CrossAnalysis) -> CrossAnalysisDto {
    CrossAnalysisDto {
        value_discovery: ca.value_discovery,
        demand_signal: ca.demand_signal,
        ground_safety: ca.ground_safety,
    }
}

impl TlsResponse {
    /// Construct a TLS response from handler coordinates and usecase output.
    pub fn new(lat: f64, lng: f64, t: TlsOutput) -> Self {
        Self {
            location: LocationDto { lat, lng },
            tls: TlsSummaryDto {
                score: t.score,
                grade: t.grade.as_str(),
                label: t.grade.label(),
            },
            axes: axes_to_dto(t.axes),
            cross_analysis: cross_analysis_to_dto(t.cross_analysis),
            metadata: TlsMetadataDto {
                calculated_at: chrono::Utc::now().to_rfc3339(),
                weight_preset: serde_json::to_value(t.weight_preset)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_else(|| "balance".to_string()),
                data_freshness: t.data_freshness,
                disclaimer: SCORE_DISCLAIMER.to_string(),
            },
        }
    }
}

/// Convert a `Point` geometry inside a [`FeatureDto`] to a small `Polygon` square.
///
/// Land price data is stored as point geometries. For better visual discoverability
/// on a MapLibre GL map (especially at higher zoom levels), each point is replaced
/// with a ~30m × 30m square polygon generated by [`point_to_polygon`].
///
/// If the feature's geometry type is not `"Point"`, or if the coordinate array
/// cannot be parsed as `[lng, lat]`, the feature is left unchanged.
pub fn point_feature_to_polygon(feature: &mut FeatureDto) {
    if feature.geometry.r#type != "Point" {
        return;
    }

    // Coordinates for a Point are a JSON array [lng, lat].
    let coords = feature.geometry.coordinates.as_array();
    let (lng, lat) = match coords {
        Some(arr) if arr.len() >= 2 => {
            let lng = arr[0].as_f64();
            let lat = arr[1].as_f64();
            match (lng, lat) {
                (Some(lng), Some(lat)) => (lng, lat),
                _ => return,
            }
        }
        _ => return,
    };

    let ring = point_to_polygon(lng, lat);
    // GeoJSON Polygon coordinates: array of rings, each ring is array of [lng, lat] positions.
    let ring_json: Vec<serde_json::Value> = ring
        .iter()
        .map(|[x, y]| serde_json::json!([x, y]))
        .collect();

    feature.geometry.r#type = "Polygon".to_string();
    feature.geometry.coordinates = serde_json::json!([ring_json]);
}

/// Convert a domain [`GeoFeature`] to a [`FeatureDto`] for JSON serialization.
///
/// Bridges the domain entity to the lib's domain-independent DTO.
pub fn geo_feature_to_dto(f: crate::domain::entity::GeoFeature) -> FeatureDto {
    FeatureDto::new(f.geometry.r#type, f.geometry.coordinates, f.properties)
}

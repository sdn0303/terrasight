/// GeoJSON Feature in domain representation.
///
/// Corresponds to PostGIS `ST_AsGeoJSON` output. Coordinates follow
/// RFC 7946 `[longitude, latitude]` order.
///
/// Note: `serde_json::Value` is an allowed dependency in Domain layer
/// (data-representation library, not an I/O framework).
#[derive(Debug, Clone)]
pub struct GeoFeature {
    pub geometry: GeoJsonGeometry,
    pub properties: serde_json::Value,
}

/// GeoJSON geometry (flexible via `serde_json::Value` for coordinates).
#[derive(Debug, Clone)]
pub struct GeoJsonGeometry {
    pub r#type: String,
    pub coordinates: serde_json::Value,
}

/// Land price record for scoring and trend calculations.
#[derive(Debug, Clone)]
pub struct PriceRecord {
    pub year: i32,
    pub price_per_sqm: i64,
    /// Nearest observation point address (used in future score detail response).
    #[allow(dead_code)]
    pub address: String,
    /// Distance in meters to the observation point (used in future score detail response).
    #[allow(dead_code)]
    pub distance_m: f64,
}

/// Single data point in a price trend time series.
#[derive(Debug, Clone)]
pub struct TrendPoint {
    pub year: i32,
    pub price_per_sqm: i64,
}

/// Nearest observation point metadata for trend data.
#[derive(Debug, Clone)]
pub struct TrendLocation {
    pub address: String,
    pub distance_m: f64,
}

/// Aggregated land price statistics within a bounding box.
#[derive(Debug, Clone)]
pub struct LandPriceStats {
    pub avg_per_sqm: Option<f64>,
    pub median_per_sqm: Option<f64>,
    pub min_per_sqm: Option<i64>,
    pub max_per_sqm: Option<i64>,
    pub count: i64,
}

/// Risk statistics within a bounding box.
#[derive(Debug, Clone)]
pub struct RiskStats {
    pub flood_area_ratio: f64,
    pub steep_slope_area_ratio: f64,
    pub composite_risk: f64,
}

/// Facility counts within a bounding box.
#[derive(Debug, Clone)]
pub struct FacilityStats {
    pub schools: i64,
    pub medical: i64,
}

/// Aggregated area statistics (P0 fix: moved from Usecase to Domain).
#[derive(Debug, Clone)]
pub struct AreaStats {
    pub land_price: LandPriceStats,
    pub risk: RiskStats,
    pub facilities: FacilityStats,
    pub zoning_distribution: std::collections::HashMap<String, f64>,
}

/// Health check result (P0 fix: moved from Usecase to Domain).
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: &'static str,
    pub db_connected: bool,
    pub reinfolib_key_set: bool,
    pub version: &'static str,
}

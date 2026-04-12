//! Shared constants for layer IDs, JSON property keys, and GeoJSON format strings.

// ── Layer IDs ──
// Used in spatial_index.rs (extract_stats_data) and lib.rs (compute_area_stats).
// hyphen-case = FGB/WASM canonical form.

pub(crate) const LAYER_LANDPRICE: &str = "landprice";
pub(crate) const LAYER_FLOOD_HISTORY: &str = "flood-history";
pub(crate) const LAYER_FLOOD: &str = "flood";
pub(crate) const LAYER_STEEP_SLOPE: &str = "steep-slope";
pub(crate) const LAYER_STEEP_SLOPE_ALT: &str = "steep_slope";
pub(crate) const LAYER_SCHOOLS: &str = "schools";
pub(crate) const LAYER_MEDICAL: &str = "medical";
pub(crate) const LAYER_RAILWAY: &str = "railway";
pub(crate) const LAYER_STATION: &str = "station";
pub(crate) const LAYER_ZONING: &str = "zoning";

// ── GeoJSON property keys ──
// Used when extracting feature properties in spatial_index.rs and stats.rs.

pub(crate) const PROP_PRICE_PER_SQM: &str = "price_per_sqm";
pub(crate) const PROP_ZONE_TYPE: &str = "zone_type";

// ── GeoJSON structural keys ──
// Used in fgb_reader.rs for parsing.

pub(crate) const GEOJSON_KEY_TYPE: &str = "type";
pub(crate) const GEOJSON_KEY_GEOMETRY: &str = "geometry";
pub(crate) const GEOJSON_KEY_PROPERTIES: &str = "properties";
pub(crate) const GEOJSON_KEY_COORDINATES: &str = "coordinates";
pub(crate) const GEOJSON_KEY_GEOMETRIES: &str = "geometries";

// ── GeoJSON geometry type names ──

pub(crate) const GEOJSON_TYPE_POINT: &str = "Point";
pub(crate) const GEOJSON_TYPE_LINE_STRING: &str = "LineString";
pub(crate) const GEOJSON_TYPE_POLYGON: &str = "Polygon";
pub(crate) const GEOJSON_TYPE_MULTI_POLYGON: &str = "MultiPolygon";

// ── GeoJSON FeatureCollection format ──

pub(crate) const FC_HEADER: &str = r#"{"type":"FeatureCollection","features":["#;
pub(crate) const FC_FOOTER: &str = "]}";

// ── Coordinate validation ──

pub(crate) const MIN_COORD_PAIR_LEN: usize = 2;

// ── WGS84 coordinate system limits ──

pub(crate) const LAT_MIN: f64 = -90.0;
pub(crate) const LAT_MAX: f64 = 90.0;
pub(crate) const LNG_MIN: f64 = -180.0;
pub(crate) const LNG_MAX: f64 = 180.0;

pub(crate) const LAT_RANGE: std::ops::RangeInclusive<f64> = LAT_MIN..=LAT_MAX;
pub(crate) const LNG_RANGE: std::ops::RangeInclusive<f64> = LNG_MIN..=LNG_MAX;

// ── Risk computation weights ──

pub(crate) const RISK_WEIGHT_FLOOD: f64 = 0.6;
pub(crate) const RISK_WEIGHT_STEEP: f64 = 0.4;

// ── Zoning classification keywords ──

pub(crate) const COMMERCIAL_ZONE_KEYWORD: &str = "商業";

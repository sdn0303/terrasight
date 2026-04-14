//! Shared constants for the Backend (`terrasight-api`) and WASM (`terrasight-wasm`) runtimes.
//!
//! All values here are part of the public API contract: changing them may affect
//! both the Axum query layer and the in-browser spatial engine simultaneously.
//! Every constant is `pub` so either runtime can import it directly.

// ── Layer IDs ────────────────────────────────────────────────────────────────

/// Canonical layer identifier for land price data.
///
/// Used as the MapLibre GL layer ID and as the FlatGeobuf file stem.
/// The canonical form strips hyphens and underscores (e.g. `"land-price"` →
/// `"landprice"`). Use [`crate`]-level `canonicalLayerId()` at UI/WASM
/// boundaries when converting from kebab-case UI IDs.
pub const LAYER_LANDPRICE: &str = "landprice";

/// Canonical layer identifier for historical flood inundation records.
pub const LAYER_FLOOD_HISTORY: &str = "floodhistory";

/// Canonical layer identifier for designated flood hazard zones.
pub const LAYER_FLOOD: &str = "flood";

/// Canonical layer identifier for steep-slope (急傾斜地) hazard zones.
pub const LAYER_STEEP_SLOPE: &str = "steepslope";

/// Canonical layer identifier for zoning (用途地域) polygons.
pub const LAYER_ZONING: &str = "zoning";

/// Canonical layer identifier for school point data.
pub const LAYER_SCHOOLS: &str = "schools";

/// Canonical layer identifier for medical facility point data.
pub const LAYER_MEDICAL: &str = "medical";

// ── GeoJSON property keys ─────────────────────────────────────────────────────

/// GeoJSON feature property key for land price per square metre (円/㎡).
///
/// Used in both PostGIS queries (`SELECT price_per_sqm`) and MapLibre GL
/// `data-driven` styling expressions. Must match the serialised field name in
/// the backend DTO exactly.
pub const PROP_PRICE_PER_SQM: &str = "price_per_sqm";

/// GeoJSON feature property key for the zoning classification string.
///
/// Values follow the MLIT 用途地域 code strings (e.g. `"第一種低層住居専用地域"`).
pub const PROP_ZONE_TYPE: &str = "zone_type";

// ── Stats risk weights ────────────────────────────────────────────────────────

/// Weight applied to flood area ratio when computing [`crate::types::RiskStats::composite_risk`].
///
/// Composite risk = `STATS_RISK_WEIGHT_FLOOD × flood_area_ratio
///                 + STATS_RISK_WEIGHT_STEEP × steep_slope_area_ratio`.
/// Both weights must sum to `1.0`.
pub const STATS_RISK_WEIGHT_FLOOD: f64 = 0.6;

/// Weight applied to steep-slope area ratio when computing [`crate::types::RiskStats::composite_risk`].
///
/// See [`STATS_RISK_WEIGHT_FLOOD`] for the full formula.
pub const STATS_RISK_WEIGHT_STEEP: f64 = 0.4;

// ── Prefecture code validation ────────────────────────────────────────────────

/// Minimum valid JIS X 0401 prefecture code (北海道 = 1).
pub const PREF_CODE_MIN: u8 = 1;

/// Maximum valid JIS X 0401 prefecture code (沖縄県 = 47).
pub const PREF_CODE_MAX: u8 = 47;

/// Fixed display width of a prefecture code string (zero-padded, e.g. `"01"`).
pub const PREF_CODE_LEN: usize = 2;

// ── Coordinate bounds ─────────────────────────────────────────────────────────

/// Maximum valid latitude in decimal degrees (WGS 84).
pub const LAT_MAX: f64 = 90.0;

/// Maximum valid longitude in decimal degrees (WGS 84).
pub const LNG_MAX: f64 = 180.0;

/// Maximum allowed side length of a bounding-box query in decimal degrees.
///
/// Requests with a wider bbox are rejected at the handler boundary to prevent
/// PostGIS full-table scans. At Tokyo's latitude, `0.5°` is roughly 44 km
/// east-west and 55 km north-south — well above any reasonable viewport.
pub const BBOX_MAX_SIDE_DEG: f64 = 0.5;

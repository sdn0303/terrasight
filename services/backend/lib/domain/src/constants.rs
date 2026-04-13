//! Shared constants for Backend and WASM.

// ── Layer IDs (canonical form: hyphens/underscores removed) ──
pub const LAYER_LANDPRICE: &str = "landprice";
pub const LAYER_FLOOD_HISTORY: &str = "floodhistory";
pub const LAYER_FLOOD: &str = "flood";
pub const LAYER_STEEP_SLOPE: &str = "steepslope";
pub const LAYER_ZONING: &str = "zoning";
pub const LAYER_SCHOOLS: &str = "schools";
pub const LAYER_MEDICAL: &str = "medical";

// ── GeoJSON property keys ──
pub const PROP_PRICE_PER_SQM: &str = "price_per_sqm";
pub const PROP_ZONE_TYPE: &str = "zone_type";

// ── Stats risk weights ──
pub const STATS_RISK_WEIGHT_FLOOD: f64 = 0.6;
pub const STATS_RISK_WEIGHT_STEEP: f64 = 0.4;

// ── Prefecture code validation ──
pub const PREF_CODE_MIN: u8 = 1;
pub const PREF_CODE_MAX: u8 = 47;
pub const PREF_CODE_LEN: usize = 2;

// ── Coordinate bounds ──
pub const LAT_MAX: f64 = 90.0;
pub const LNG_MAX: f64 = 180.0;
pub const BBOX_MAX_SIDE_DEG: f64 = 0.5;

//! Named constants for scoring algorithms and business rules.
//!
//! All magic numbers used across the scoring, spatial search, and validation
//! logic are centralized here to make thresholds easy to audit and adjust.

/// Maximum score contribution from a single scoring component.
/// Each of the four components (trend, risk, access, yield) contributes at most this value,
/// giving a combined maximum of 100.
pub const SCORE_COMPONENT_MAX: f64 = 25.0;

// ---------------------------------------------------------------------------
// Trend scoring
// ---------------------------------------------------------------------------

/// Multiplier applied to the CAGR value when converting it to a score in
/// the 0–[`SCORE_COMPONENT_MAX`] range.
///
/// score = clamp(cagr * TREND_CAGR_MULTIPLIER, 0, SCORE_COMPONENT_MAX)
pub const TREND_CAGR_MULTIPLIER: f64 = 500.0;

// ---------------------------------------------------------------------------
// Risk scoring — point-based (single coordinate query)
// ---------------------------------------------------------------------------

/// Weight assigned to flood risk when computing the point-based risk score.
pub const RISK_WEIGHT_FLOOD: f64 = 0.4;

/// Weight assigned to liquefaction risk when computing the point-based risk score.
///
/// Phase 1: the input value is always 0.0 (liquefaction data not yet ingested),
/// so this weight has no practical effect until Phase 2 data is available.
pub const RISK_WEIGHT_LIQUEFACTION: f64 = 0.4;

/// Weight assigned to steep-slope risk when computing the point-based risk score.
pub const RISK_WEIGHT_STEEP: f64 = 0.2;

// ---------------------------------------------------------------------------
// Risk scoring — area-based statistics (bbox / polygon query)
// ---------------------------------------------------------------------------

/// Flood risk weight used in the area-based stats risk calculation.
///
/// Intentionally differs from [`RISK_WEIGHT_FLOOD`] because the area-based
/// query aggregates over many cells and omits the liquefaction component.
#[expect(
    dead_code,
    reason = "consumed by the stats usecase in a subsequent task"
)]
pub const STATS_RISK_WEIGHT_FLOOD: f64 = 0.6;

/// Steep-slope risk weight used in the area-based stats risk calculation.
#[expect(
    dead_code,
    reason = "consumed by the stats usecase in a subsequent task"
)]
pub const STATS_RISK_WEIGHT_STEEP: f64 = 0.4;

// ---------------------------------------------------------------------------
// Access scoring
// ---------------------------------------------------------------------------

/// Number of schools within search radius at which the school sub-score
/// saturates (i.e. reaches [`ACCESS_SCHOOL_MAX_SCORE`]).
pub const ACCESS_SCHOOL_SATURATION: f64 = 3.0;

/// Maximum score contribution from school accessibility.
pub const ACCESS_SCHOOL_MAX_SCORE: f64 = 10.0;

/// Number of medical facilities within search radius at which the medical
/// sub-score saturates (i.e. reaches [`ACCESS_MEDICAL_MAX_SCORE`]).
pub const ACCESS_MEDICAL_SATURATION: f64 = 5.0;

/// Maximum score contribution from medical facility accessibility.
pub const ACCESS_MEDICAL_MAX_SCORE: f64 = 10.0;

/// Divisor applied to the nearest-station distance (in metres) when computing
/// the transit distance bonus.
///
/// bonus = clamp(ACCESS_DISTANCE_DIVISOR / distance_m, 0, ACCESS_DISTANCE_MAX_BONUS)
pub const ACCESS_DISTANCE_DIVISOR: f64 = 200.0;

/// Maximum bonus score awarded for proximity to the nearest transit station.
pub const ACCESS_DISTANCE_MAX_BONUS: f64 = 5.0;

// ---------------------------------------------------------------------------
// Yield scoring
// ---------------------------------------------------------------------------

/// Fraction of the median transaction price used as the estimated annual rent
/// proxy when no direct rental data is available.
pub const YIELD_TRANSACTION_RATIO: f64 = 0.8;

/// Multiplier applied to the gross yield ratio when converting it to a score
/// in the 0–[`SCORE_COMPONENT_MAX`] range.
pub const YIELD_SCORE_MULTIPLIER: f64 = 500.0;

// ---------------------------------------------------------------------------
// Spatial search radii (metres)
// ---------------------------------------------------------------------------

/// Radius (m) used for buffered PostGIS queries when fetching risk data
/// around a point of interest.
#[expect(
    dead_code,
    reason = "consumed by the infra repository layer in a subsequent task"
)]
pub const RADIUS_RISK_BUFFER_M: f64 = 500.0;

/// Radius (m) used when searching for nearby schools and medical facilities.
#[expect(
    dead_code,
    reason = "consumed by the infra repository layer in a subsequent task"
)]
pub const RADIUS_FACILITY_SEARCH_M: f64 = 1000.0;

/// Radius (m) used when aggregating historical transaction data for trend
/// analysis around a given coordinate.
#[expect(
    dead_code,
    reason = "consumed by the infra repository layer in a subsequent task"
)]
pub const RADIUS_TREND_SEARCH_M: f64 = 2000.0;

// ---------------------------------------------------------------------------
// Trend analysis window
// ---------------------------------------------------------------------------

/// Default number of years of historical data used for CAGR calculation
/// when the caller does not specify an explicit window.
#[expect(
    dead_code,
    reason = "consumed by the handler request extractor in a subsequent task"
)]
pub const TREND_DEFAULT_YEARS: i32 = 5;

/// Minimum allowed value for the trend analysis year window.
pub const TREND_MIN_YEARS: i32 = 1;

/// Maximum allowed value for the trend analysis year window.
pub const TREND_MAX_YEARS: i32 = 20;

// ---------------------------------------------------------------------------
// Coordinate validation bounds
// ---------------------------------------------------------------------------

/// Maximum absolute latitude value (degrees). Coordinates with |lat| > this
/// value are rejected as invalid.
pub const LAT_MAX: f64 = 90.0;

/// Maximum absolute longitude value (degrees). Coordinates with |lng| > this
/// value are rejected as invalid.
pub const LNG_MAX: f64 = 180.0;

/// Maximum allowed side length of a bounding box (degrees). Requests for
/// bounding boxes larger than this are rejected to prevent runaway queries.
pub const BBOX_MAX_SIDE_DEG: f64 = 0.5;

// ---------------------------------------------------------------------------
// Decimal precision (number of fractional digits after rounding)
// ---------------------------------------------------------------------------

/// Number of decimal places used when rounding final investment scores.
pub const PRECISION_SCORE: u32 = 1;

/// Number of decimal places used when rounding ratio values (e.g. gross yield).
pub const PRECISION_RATIO: u32 = 3;

/// Number of decimal places used when rounding distance values (metres).
pub const PRECISION_DISTANCE: u32 = 1;

// ---------------------------------------------------------------------------
// SQL / PostGIS
// ---------------------------------------------------------------------------

/// Percentile argument passed to PostGIS `percentile_cont` for median
/// calculations (0.5 = 50th percentile).
#[expect(
    dead_code,
    reason = "consumed by the infra repository layer in a subsequent task"
)]
pub const MEDIAN_PERCENTILE: f64 = 0.5;

/// EPSG SRID for the WGS 84 geographic coordinate system used in all
/// GeoJSON output and PostGIS geometry storage.
#[expect(
    dead_code,
    reason = "consumed by the infra repository layer in a subsequent task"
)]
pub const SRID_WGS84: i32 = 4326;

// ---------------------------------------------------------------------------
// Health status strings
// ---------------------------------------------------------------------------

/// Health check response body `status` value when all subsystems are healthy.
pub const HEALTH_STATUS_OK: &str = "ok";

/// Health check response body `status` value when one or more subsystems are
/// reachable but operating in a degraded state.
pub const HEALTH_STATUS_DEGRADED: &str = "degraded";

// ---------------------------------------------------------------------------
// User-facing text
// ---------------------------------------------------------------------------

/// Disclaimer appended to all investment score responses, reminding users
/// that the score is informational and not financial advice.
#[expect(
    dead_code,
    reason = "consumed by the score handler response in a subsequent task"
)]
pub const SCORE_DISCLAIMER: &str = "本スコアは参考値です。投資判断は自己責任で行ってください。";

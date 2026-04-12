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
pub const RISK_WEIGHT_FLOOD: f64 = 0.25;

/// Weight assigned to seismic hazard (30-year probability of intensity ≥ 6弱)
/// when computing the point-based risk score.
pub const RISK_WEIGHT_SEISMIC: f64 = 0.30;

/// Weight assigned to steep-slope risk when computing the point-based risk score.
pub const RISK_WEIGHT_STEEP: f64 = 0.15;

/// Weight assigned to ground amplification (derived from AVS30) when computing
/// the point-based risk score.
pub const RISK_WEIGHT_GROUND_AMP: f64 = 0.30;

/// AVS30 threshold (m/s) above which ground is classified as very firm.
///
/// Sites with AVS30 > this value receive a ground-amplification factor of 0.0.
pub const RISK_AVS30_FIRM: f64 = 400.0;

/// AVS30 threshold (m/s) below which ground is classified as soft.
///
/// Sites with AVS30 < this value receive a ground-amplification factor of
/// [`RISK_GROUND_AMP_SOFT`].
pub const RISK_AVS30_SOFT: f64 = 200.0;

/// Ground amplification factor applied when AVS30 falls in the moderate range
/// (200–400 m/s).
pub const RISK_GROUND_AMP_MODERATE: f64 = 0.3;

/// Ground amplification factor applied when AVS30 is below [`RISK_AVS30_SOFT`]
/// (soft ground with high seismic amplification risk).
pub const RISK_GROUND_AMP_SOFT: f64 = 0.8;

// ---------------------------------------------------------------------------
// Risk scoring — area-based statistics (bbox / polygon query)
// ---------------------------------------------------------------------------

/// Flood risk weight used in the area-based stats risk calculation.
///
/// Intentionally differs from [`RISK_WEIGHT_FLOOD`] because the area-based
/// query aggregates over many cells and omits the liquefaction component.
pub const STATS_RISK_WEIGHT_FLOOD: f64 = 0.6;

/// Steep-slope risk weight used in the area-based stats risk calculation.
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
pub const RADIUS_RISK_BUFFER_M: f64 = 500.0;

/// Radius (m) used when searching for nearby schools and medical facilities.
pub const RADIUS_FACILITY_SEARCH_M: f64 = 1000.0;

/// Radius (m) used when aggregating historical transaction data for trend
/// analysis around a given coordinate.
pub const RADIUS_TREND_SEARCH_M: f64 = 2000.0;

// ---------------------------------------------------------------------------
// Trend analysis window
// ---------------------------------------------------------------------------

/// Default number of years of historical data used for CAGR calculation
/// when the caller does not specify an explicit window.
pub const TREND_DEFAULT_YEARS: i32 = 5;

/// Minimum allowed value for the trend analysis year window.
pub const TREND_MIN_YEARS: i32 = 1;

/// Maximum allowed value for the trend analysis year window.
pub const TREND_MAX_YEARS: i32 = 20;

// ---------------------------------------------------------------------------
// Year validation bounds (for land price queries)
// ---------------------------------------------------------------------------

/// Minimum valid year for land price data queries.
pub const YEAR_MIN: i32 = 2000;

/// Maximum valid year for land price data queries.
pub const YEAR_MAX: i32 = 2100;

// ---------------------------------------------------------------------------
// Prefecture code validation
// ---------------------------------------------------------------------------

/// Length (in ASCII digits) of a valid prefecture code.
pub(crate) const PREF_CODE_LEN: usize = 2;

/// Minimum valid prefecture code value (Hokkaido = 01).
pub(crate) const PREF_CODE_MIN: u8 = 1;

/// Maximum valid prefecture code value (Okinawa = 47).
pub(crate) const PREF_CODE_MAX: u8 = 47;

// ---------------------------------------------------------------------------
// City code validation
// ---------------------------------------------------------------------------

/// Length (in ASCII digits) of a valid JIS X 0402 city code.
pub(crate) const CITY_CODE_LEN: usize = 5;

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
pub const MEDIAN_PERCENTILE: f64 = 0.5;

/// EPSG SRID for the WGS 84 geographic coordinate system used in all
/// GeoJSON output and PostGIS geometry storage.
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
pub const SCORE_DISCLAIMER: &str = "本スコアは参考値です。投資判断は自己責任で行ってください。";

// ---------------------------------------------------------------------------
// Opportunities endpoint (/api/v1/opportunities)
// ---------------------------------------------------------------------------

/// Default `limit` for the opportunities endpoint.
pub const DEFAULT_OPPORTUNITY_LIMIT: u32 = 50;

/// Maximum server-enforced `limit` for the opportunities endpoint.
pub const MAX_OPPORTUNITY_LIMIT: u32 = 50;

/// End-to-end request timeout for the opportunities endpoint (seconds).
pub const OPPORTUNITY_TIMEOUT_SECS: u64 = 8;

/// Per-query timeout for individual SQL calls on the opportunities path.
pub const OPPORTUNITY_QUERY_TIMEOUT_SECS: u64 = 5;

/// Cache TTL for the opportunities response (seconds).
pub const OPPORTUNITY_CACHE_TTL_SECS: u64 = 60;

/// Maximum in-memory cache entries for opportunities responses.
pub const OPPORTUNITY_CACHE_MAX_ENTRIES: u64 = 256;

/// Maximum parallelism for per-record TLS computation.
pub const OPPORTUNITY_TLS_CONCURRENCY: usize = 4;

/// Size of the raw record pool fetched from the DB per `/api/v1/opportunities`
/// request.
///
/// The usecase fetches this many records (with `offset = 0`) from
/// `LandPriceRepository::find_for_opportunities`, runs TLS enrichment
/// plus `tls_min`/`risk_max` filtering on the full pool, and caches
/// the result. User-facing `limit`/`offset` pagination is then applied
/// after the cache by slicing the cached pool in-memory.
///
/// This design guarantees that `limit` is honoured even when
/// `tls_min`/`risk_max` reject most raw records, cache entries are
/// shared across pagination pages for the same filter set, and TLS
/// compute cost per cache miss is bounded to
/// `OPPORTUNITY_FETCH_POOL_SIZE` records.
///
/// The trade-off is that pagination is limited to the first
/// `OPPORTUNITY_FETCH_POOL_SIZE` records after filtering — `offset`
/// values beyond that return empty results.
pub const OPPORTUNITY_FETCH_POOL_SIZE: u32 = 100;

// ---------------------------------------------------------------------------
// Transaction endpoint (/api/v1/transactions)
// ---------------------------------------------------------------------------

/// Default `limit` for the transactions list endpoint.
pub const DEFAULT_TRANSACTION_LIMIT: u32 = 50;

/// Maximum server-enforced `limit` for the transactions list endpoint.
pub const MAX_TRANSACTION_LIMIT: u32 = 200;

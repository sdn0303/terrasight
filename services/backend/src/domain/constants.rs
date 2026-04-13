//! Named constants for scoring algorithms and business rules.
//!
//! All magic numbers used across the scoring, spatial search, and validation
//! logic are centralized here to make thresholds easy to audit and adjust.

// ---------------------------------------------------------------------------
// Risk scoring — area-based statistics (bbox / polygon query)
// ---------------------------------------------------------------------------

/// Flood risk weight used in the area-based stats risk calculation.
///
/// Area-based queries aggregate over many cells and omit the liquefaction component.
pub(crate) const STATS_RISK_WEIGHT_FLOOD: f64 = 0.6;

/// Steep-slope risk weight used in the area-based stats risk calculation.
pub(crate) const STATS_RISK_WEIGHT_STEEP: f64 = 0.4;

// ---------------------------------------------------------------------------
// Spatial search radii (metres)
// ---------------------------------------------------------------------------

/// Radius (m) used when aggregating historical transaction data for trend
/// analysis around a given coordinate.
pub(crate) const RADIUS_TREND_SEARCH_M: f64 = 2000.0;

// ---------------------------------------------------------------------------
// Trend analysis window
// ---------------------------------------------------------------------------

/// Default number of years of historical data used for CAGR calculation
/// when the caller does not specify an explicit window.
pub(crate) const TREND_DEFAULT_YEARS: i32 = 5;

/// Minimum allowed value for the trend analysis year window.
pub(crate) const TREND_MIN_YEARS: i32 = 1;

/// Maximum allowed value for the trend analysis year window.
pub(crate) const TREND_MAX_YEARS: i32 = 20;

// ---------------------------------------------------------------------------
// Year validation bounds (for land price queries)
// ---------------------------------------------------------------------------

/// Minimum valid year for land price data queries.
pub(crate) const YEAR_MIN: i32 = 2000;

/// Maximum valid year for land price data queries.
pub(crate) const YEAR_MAX: i32 = 2100;

// ---------------------------------------------------------------------------
// Coordinate validation bounds
// ---------------------------------------------------------------------------

/// Maximum absolute latitude value (degrees). Coordinates with |lat| > this
/// value are rejected as invalid.
pub(crate) const LAT_MAX: f64 = 90.0;

/// Maximum absolute longitude value (degrees). Coordinates with |lng| > this
/// value are rejected as invalid.
pub(crate) const LNG_MAX: f64 = 180.0;

/// Maximum allowed side length of a bounding box (degrees). Requests for
/// bounding boxes larger than this are rejected to prevent runaway queries.
pub(crate) const BBOX_MAX_SIDE_DEG: f64 = 0.5;

// ---------------------------------------------------------------------------
// Decimal precision (number of fractional digits after rounding)
// ---------------------------------------------------------------------------

/// Number of decimal places used when rounding ratio values (e.g. gross yield).
pub(crate) const PRECISION_RATIO: u32 = 3;

// ---------------------------------------------------------------------------
// Health status strings
// ---------------------------------------------------------------------------

/// Health check response body `status` value when all subsystems are healthy.
pub(crate) const HEALTH_STATUS_OK: &str = "ok";

/// Health check response body `status` value when one or more subsystems are
/// reachable but operating in a degraded state.
pub(crate) const HEALTH_STATUS_DEGRADED: &str = "degraded";

// ---------------------------------------------------------------------------
// User-facing text
// ---------------------------------------------------------------------------

/// Disclaimer appended to all investment score responses, reminding users
/// that the score is informational and not financial advice.
pub(crate) const SCORE_DISCLAIMER: &str =
    "本スコアは参考値です。投資判断は自己責任で行ってください。";

// ---------------------------------------------------------------------------
// Opportunities endpoint (/api/v1/opportunities)
// ---------------------------------------------------------------------------

/// Default `limit` for the opportunities endpoint.
pub(crate) const DEFAULT_OPPORTUNITY_LIMIT: u32 = 50;

/// Maximum server-enforced `limit` for the opportunities endpoint.
pub(crate) const MAX_OPPORTUNITY_LIMIT: u32 = 50;

/// End-to-end request timeout for the opportunities endpoint (seconds).
pub(crate) const OPPORTUNITY_TIMEOUT_SECS: u64 = 8;

/// Per-query timeout for individual SQL calls on the opportunities path.
pub(crate) const OPPORTUNITY_QUERY_TIMEOUT_SECS: u64 = 5;

/// Cache TTL for the opportunities response (seconds).
pub(crate) const OPPORTUNITY_CACHE_TTL_SECS: u64 = 60;

/// Maximum in-memory cache entries for opportunities responses.
pub(crate) const OPPORTUNITY_CACHE_MAX_ENTRIES: u64 = 256;

/// Maximum parallelism for per-record TLS computation.
pub(crate) const OPPORTUNITY_TLS_CONCURRENCY: usize = 4;

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
pub(crate) const OPPORTUNITY_FETCH_POOL_SIZE: u32 = 100;

// ---------------------------------------------------------------------------
// Prefecture / City code validation
// ---------------------------------------------------------------------------

/// Minimum valid prefecture code number (Hokkaido = 01).
pub(crate) const PREF_CODE_MIN: u8 = 1;

/// Maximum valid prefecture code number (Okinawa = 47).
pub(crate) const PREF_CODE_MAX: u8 = 47;

/// Expected string length of a prefecture code (zero-padded 2 digits).
pub(crate) const PREF_CODE_LEN: usize = 2;

// ---------------------------------------------------------------------------
// Entity validation bounds
// ---------------------------------------------------------------------------

/// Minimum valid building coverage ratio (建蔽率) percentage.
pub(crate) const BCR_MIN: i32 = 0;

/// Maximum valid building coverage ratio (建蔽率) percentage.
pub(crate) const BCR_MAX: i32 = 100;

/// Minimum valid floor area ratio (容積率) percentage.
pub(crate) const FAR_MIN: i32 = 0;

/// Maximum valid floor area ratio (容積率) percentage.
pub(crate) const FAR_MAX: i32 = 2000;

// ---------------------------------------------------------------------------
// Handler defaults
// ---------------------------------------------------------------------------

/// Default `from` year for the land-price year-range endpoint.
pub(crate) const DEFAULT_FROM_YEAR: i32 = 2019;

/// Default `to` year for the land-price year-range endpoint.
pub(crate) const DEFAULT_TO_YEAR: i32 = 2024;

/// Default map zoom level used when the client omits the parameter.
pub(crate) const DEFAULT_ZOOM_LEVEL: u32 = 14;

// ---------------------------------------------------------------------------
// External API timeouts
// ---------------------------------------------------------------------------

/// Timeout (seconds) for J-SHIS (Japan Seismic Hazard Information Station) API requests.
pub(crate) const JSHIS_TIMEOUT_SECS: u64 = 30;

// ---------------------------------------------------------------------------
// TLS query search radii (metres) — per-query overrides of the shared radii
// ---------------------------------------------------------------------------

/// Search radius (m) for nearest land-price records in TLS queries.
pub(crate) const TLS_PRICE_SEARCH_RADIUS_M: f64 = 1000.0;

/// Search radius (m) for flood-risk and steep-slope lookups in TLS queries.
pub(crate) const TLS_RISK_SEARCH_RADIUS_M: f64 = 500.0;

/// Search radius (m) for school proximity lookups in TLS queries.
pub(crate) const TLS_SCHOOL_SEARCH_RADIUS_M: f64 = 800.0;

/// Search radius (m) for transaction-count lookups in TLS queries.
pub(crate) const TLS_TRANSACTION_SEARCH_RADIUS_M: f64 = 500.0;

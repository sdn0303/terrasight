//! Metric name constants for the Real Estate Investment API.
//!
//! All metric names follow the pattern: `{subsystem}.{resource}.{action}`.
//! This naming convention enables Prometheus/Grafana grouping and filtering.
//!
//! Metric types:
//! - `_total` suffix → counter (monotonically increasing)
//! - `_seconds` suffix → histogram (request/query duration)
//! - No suffix → gauge (current value)

// ─── HTTP API ────────────────────────────────────────────────────────

/// Total HTTP requests received (counter).
/// Labels: endpoint, method, status
pub const API_REQUEST_TOTAL: &str = "api.request.total";

/// HTTP request duration in seconds (histogram).
/// Labels: endpoint, method
pub const API_REQUEST_DURATION_SECONDS: &str = "api.request.duration_seconds";

/// Total HTTP error responses (counter).
/// Labels: endpoint, status, error_kind
pub const API_ERROR_TOTAL: &str = "api.error.total";

// ─── Database ────────────────────────────────────────────────────────

/// Database query duration in seconds (histogram).
/// Labels: query_type, db_op
pub const DB_QUERY_DURATION_SECONDS: &str = "db.query.duration_seconds";

/// Total database query errors (counter).
/// Labels: query_type, error_kind
pub const DB_QUERY_ERROR_TOTAL: &str = "db.query.error_total";

/// Current active database pool connections (gauge).
pub const DB_POOL_ACTIVE: &str = "db.pool.active";

/// Current idle database pool connections (gauge).
pub const DB_POOL_IDLE: &str = "db.pool.idle";

// ─── Spatial / PostGIS ───────────────────────────────────────────────
//
// TODO(user): Define the PostGIS-specific metrics below.
//
// Consider what spatial query characteristics matter for monitoring:
//
// 1. Bounding box request area (histogram) — tracks how large the bbox
//    queries are.  Large bbox = expensive ST_Intersects scan.
//    Suggested: "spatial.bbox.area_deg2"
//
// 2. GeoJSON features returned per request (histogram) — tracks response
//    payload size.  Spikes may indicate missing spatial indexes.
//    Suggested: "spatial.features.returned"
//
// 3. Nearest-point search distance (histogram) — for trend/score endpoints
//    that use ST_DWithin.  Tracks how far the nearest data point is.
//    Suggested: "spatial.nearest.distance_m"
//
// 4. Layer query count (counter) — which layers are most requested?
//    Suggested: "spatial.layer.query_total"
//
// Fill in the constants below with your chosen metric names.
// The naming pattern is: `spatial.{resource}.{measurement}`.

/// Bounding box area in square degrees (histogram).
/// Labels: endpoint
pub const SPATIAL_BBOX_AREA_DEG2: &str = "spatial.bbox.area_deg2";

/// Number of GeoJSON features returned per layer query (histogram).
/// Labels: layer
pub const SPATIAL_FEATURES_RETURNED: &str = "spatial.features.returned";

/// Distance to nearest observation point in meters (histogram).
/// Labels: endpoint
pub const SPATIAL_NEAREST_DISTANCE_M: &str = "spatial.nearest.distance_m";

/// Total spatial layer queries (counter).
/// Labels: layer
pub const SPATIAL_LAYER_QUERY_TOTAL: &str = "spatial.layer.query_total";

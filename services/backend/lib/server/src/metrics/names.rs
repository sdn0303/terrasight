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

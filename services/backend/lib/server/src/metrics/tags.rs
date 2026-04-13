//! Tag (label) name constants for metric dimensions.
//!
//! Using constants prevents typos and enables IDE autocompletion across
//! all crates that emit metrics.
//!
//! # Usage
//!
//! ```rust,ignore
//! use terrasight_server::metrics::{counter, tags};
//!
//! counter!(names::API_REQUEST_TOTAL, tags::ENDPOINT => "area-data", tags::METHOD => "GET")
//!     .increment(1);
//! ```

/// HTTP method (`GET`, `POST`, …).
pub const METHOD: &str = "method";

/// API endpoint path (e.g., `/api/area-data`).
pub const ENDPOINT: &str = "endpoint";

/// HTTP response status code class (`2xx`, `4xx`, `5xx`).
pub const STATUS: &str = "status";

/// Area data layer type (`landprice`, `zoning`, `flood`, …).
pub const LAYER: &str = "layer";

/// Type of spatial query (`st_intersects`, `st_dwithin`, …).
pub const QUERY_TYPE: &str = "query_type";

/// Deployment environment (`dev`, `staging`, `prod`).
pub const ENV: &str = "env";

/// Database operation type (`select`, `insert`, `update`).
pub const DB_OP: &str = "db_op";

/// Error classification for error counters.
pub const ERROR_KIND: &str = "error_kind";

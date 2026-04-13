//! # terrasight-server
//!
//! Server infrastructure crate: DB pool, HTTP middleware, and telemetry.
//!
//! Merges `realestate-api-core`, `realestate-db`, and `realestate-telemetry`
//! into a single crate with feature flags.
//!
//! ## Feature Flags
//!
//! | Feature   | Default | Description |
//! |-----------|---------|-------------|
//! | `db`      | ✅      | PostgreSQL pool, spatial helpers, error mapping |
//! | `http`    | ✅      | Axum middleware: rate limit, response time, request ID, tracing |
//! | `metrics` | ✅      | Prometheus-compatible metrics (counters, histograms, gauges) |
//! | `log`     | ✅      | Structured logging subscriber with dev/prod format switching |

#[cfg(feature = "db")]
pub mod db;

#[cfg(feature = "http")]
pub mod http;

#[cfg(feature = "log")]
pub mod log;

#[cfg(feature = "metrics")]
pub mod metrics;

/// Re-export `tracing` so consumers can instrument code without adding
/// `tracing` as a direct dependency.
pub use tracing;

//! # realestate-telemetry
//!
//! Observability toolkit for the Real Estate Investment Data Visualizer API.
//!
//! This crate provides a thin initialization and shared vocabulary layer for
//! structured logging, application metrics, and HTTP request tracing.
//!
//! ## Feature Flags
//!
//! | Feature   | Default | Description |
//! |-----------|---------|-------------|
//! | `log`     | ✅      | Structured logging subscriber with dev/prod format switching |
//! | `metrics` | ❌      | Prometheus-compatible metrics (counters, histograms, gauges) |
//! | `http`    | ❌      | Tower-HTTP trace layer for Axum request/response logging |
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use realestate_telemetry::{log, http, metrics};
//!
//! // Initialize logging (call once at startup)
//! log::init_global_logger(log::LogFormat::Pretty, None);
//!
//! // Start Prometheus metrics endpoint (background task)
//! metrics::init_prometheus("0.0.0.0:9090").await?;
//!
//! // Build Axum router with tracing layer
//! let app = Router::new()
//!     .route("/api/health", get(health))
//!     .layer(http::trace_layer());
//! ```

#[cfg(feature = "log")]
pub mod log;

#[cfg(feature = "metrics")]
pub mod metrics;

#[cfg(feature = "http")]
pub mod http;

/// Re-export `tracing` so consumers can instrument code without adding
/// `tracing` as a direct dependency.
pub use tracing;

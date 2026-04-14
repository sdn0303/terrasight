//! HTTP infrastructure: error mapping, middleware (rate limit, response time, request ID), tracing, and GeoJSON response types.
//!
//! This module provides the Axum-facing HTTP plumbing used by all handlers in the
//! Terrasight backend.  It is enabled by the `http` feature flag (on by default).
//!
//! ## Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`error`] | [`ErrorMapping`] trait and [`ApiError`] generic wrapper; converts domain errors to JSON HTTP responses |
//! | [`middleware`] | Tower middleware layers: IP rate limiting, response time header, request ID propagation |
//! | [`response`] | RFC 7946 GeoJSON DTOs ([`FeatureCollectionDto`], [`FeatureDto`]) for MapLibre GL |
//! | [`tracing`] | Pre-configured [`tower_http`] trace layer and request ID layers |
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use axum::Router;
//! use terrasight_server::http::{tracing, middleware::rate_limit::{rate_limit_layer, RateLimitConfig}};
//!
//! let app = Router::new()
//!     .layer(tracing::trace_layer())
//!     .layer(tracing::set_request_id_layer())
//!     .layer(rate_limit_layer(&RateLimitConfig { requests_per_minute: 60, burst_size: 10 }));
//! ```

/// Generic API error handling utilities.
pub mod error;

/// Tower middleware: rate limiting and response time.
pub mod middleware;

/// GeoJSON response DTO types.
pub mod response;

/// HTTP tracing layer and request ID propagation.
pub mod tracing;

pub use error::{ApiError, ErrorMapping};
pub use response::{FeatureCollectionDto, FeatureDto};

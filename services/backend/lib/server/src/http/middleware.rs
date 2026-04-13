//! Axum middleware layers for rate limiting, response time tracking, and request ID propagation.
//!
//! Each submodule exposes a single constructor function that returns a ready-to-use
//! [`tower::Layer`].  Stack them on an [`axum::Router`] in the order shown below
//! (outermost to innermost, i.e., last `.layer()` call is applied first):
//!
//! ```rust,ignore
//! use axum::Router;
//! use terrasight_server::http::middleware::{
//!     rate_limit::{rate_limit_layer, RateLimitConfig},
//!     request_id::{propagate_request_id_layer, request_id_layer},
//!     response_time::response_time_layer,
//! };
//!
//! let app = Router::new()
//!     // innermost — runs last on the way in, first on the way out
//!     .layer(response_time_layer())
//!     .layer(request_id_layer())
//!     .layer(propagate_request_id_layer())
//!     // outermost — enforces rate limit before the request reaches handlers
//!     .layer(rate_limit_layer(&RateLimitConfig {
//!         requests_per_minute: 60,
//!         burst_size: 10,
//!     }));
//! ```
//!
//! ## Submodules
//!
//! | Module | Header set | Notes |
//! |--------|-----------|-------|
//! | `rate_limit` | — | 429 on exceeded burst; keyed on peer IP |
//! | `request_id` | `x-request-id` | UUID v4; delegates to [`crate::http::tracing`] |
//! | `response_time` | `x-response-time` | Wall-clock ms, e.g. `"14ms"` |

/// IP-based rate limiting using [`tower_governor`].
pub mod rate_limit;

/// Request ID injection via `x-request-id` header.
pub mod request_id;

/// Response time measurement via `X-Response-Time` header.
pub mod response_time;

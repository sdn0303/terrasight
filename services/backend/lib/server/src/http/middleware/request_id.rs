//! Request ID middleware.
//!
//! Generates a UUID v4 for each incoming request and propagates it as the
//! `x-request-id` header on both the forwarded request and the response.
//!
//! This module delegates to [`crate::http::tracing`] which provides
//! UUID-based request ID generation via `tower-http`'s built-in
//! [`MakeRequestUuid`](tower_http::request_id::MakeRequestUuid).
//!
//! # Example
//!
//! ```rust,ignore
//! use terrasight_server::http::middleware::request_id::request_id_layer;
//! use axum::Router;
//!
//! let app: Router = Router::new()
//!     // ... routes ...
//!     .layer(request_id_layer());
//! ```

pub use crate::http::tracing::propagate_request_id_layer;
pub use crate::http::tracing::set_request_id_layer as request_id_layer;

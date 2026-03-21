//! Request ID middleware.
//!
//! Generates a UUID v4 for each incoming request and propagates it as the
//! `X-Request-Id` header on both the forwarded request and the response.
//!
//! # Example
//!
//! ```rust,ignore
//! use realestate_api_core::middleware::request_id::request_id_layer;
//! use axum::Router;
//!
//! let app: Router = Router::new()
//!     // ... routes ...
//!     .layer(request_id_layer());
//! ```

use http::Request;
use tower_http::request_id::{MakeRequestId, RequestId, SetRequestIdLayer};
use uuid::Uuid;

/// Implements [`MakeRequestId`] by generating a UUID v4 string for every
/// incoming request.
#[derive(Clone)]
pub struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let id = Uuid::new_v4().to_string();
        // The UUID format is always valid as an HTTP header value, so this
        // parse cannot fail.
        let header_val = id
            .parse()
            .expect("UUID v4 string is always a valid HTTP header value");
        Some(RequestId::new(header_val))
    }
}

/// Creates a [`tower`] layer that sets the `X-Request-Id` header on every
/// request and propagates it to the response.
///
/// If the incoming request already carries an `X-Request-Id` header the
/// existing value is preserved (pass-through behaviour from
/// [`SetRequestIdLayer`]).
pub fn request_id_layer() -> SetRequestIdLayer<MakeRequestUuid> {
    SetRequestIdLayer::x_request_id(MakeRequestUuid)
}

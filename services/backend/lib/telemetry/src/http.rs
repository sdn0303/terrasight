//! HTTP tracing layer for Axum.
//!
//! Provides a pre-configured [`tower_http::trace::TraceLayer`] that emits
//! structured tracing spans for every HTTP request, plus request ID
//! propagation via the `x-request-id` header.
//!
//! # Usage
//!
//! ```rust,ignore
//! use realestate_telemetry::http;
//!
//! let app = Router::new()
//!     .route("/api/health", get(health))
//!     .layer(http::trace_layer());
//! ```

use axum::http::{HeaderName, Request};
use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultOnRequest, DefaultOnResponse, MakeSpan, TraceLayer},
};
use tracing::{Level, Span};

/// Header name used for request ID propagation.
const REQUEST_ID_HEADER: &str = "x-request-id";

/// Custom span builder that uses the request path (without query string)
/// to avoid high-cardinality span names.
#[derive(Debug, Clone)]
pub struct ApiMakeSpan;

impl<B> MakeSpan<B> for ApiMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let path = request.uri().path();
        let method = request.method().as_str();

        tracing::info_span!(
            "http_request",
            http.method = %method,
            http.path = %path,
            http.status_code = tracing::field::Empty,
        )
    }
}

/// Build a [`TraceLayer`] that logs each HTTP request as a tracing span.
///
/// Each span includes:
/// - `http.method` — request method
/// - `http.path` — request path (without query string for cleaner grouping)
/// - `http.status_code` — response status (populated on response via `record_status_on_span`)
///
/// The layer uses [`ServerErrorsAsFailures`] to classify 5xx responses as
/// failures for tracing purposes.
pub fn trace_layer() -> TraceLayer<
    SharedClassifier<ServerErrorsAsFailures>,
    ApiMakeSpan,
    DefaultOnRequest,
    DefaultOnResponse,
> {
    TraceLayer::new_for_http()
        .make_span_with(ApiMakeSpan)
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}

/// Build a [`SetRequestIdLayer`] that generates UUID v4 request IDs.
///
/// Assigns a unique `x-request-id` to every incoming request that does not
/// already carry one.  Pair with [`propagate_request_id_layer`] to echo
/// the ID back in the response.
pub fn set_request_id_layer() -> SetRequestIdLayer<MakeRequestUuid> {
    SetRequestIdLayer::new(HeaderName::from_static(REQUEST_ID_HEADER), MakeRequestUuid)
}

/// Build a [`PropagateRequestIdLayer`] that copies the `x-request-id` header
/// from request to response.
///
/// Enables clients and load balancers to correlate requests with log entries.
pub fn propagate_request_id_layer() -> PropagateRequestIdLayer {
    PropagateRequestIdLayer::new(HeaderName::from_static(REQUEST_ID_HEADER))
}

/// Convenience function to record the response status code on the current span.
///
/// Call this in an Axum middleware or `on_response` callback to populate
/// the `http.status_code` field set by [`trace_layer`].
pub fn record_status_on_span(span: &Span, status: u16) {
    span.record("http.status_code", status);
}

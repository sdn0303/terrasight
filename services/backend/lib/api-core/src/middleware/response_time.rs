//! Response time middleware.
//!
//! Measures wall-clock time from when the request is received to when the
//! response future resolves, then appends an `X-Response-Time` header with
//! the elapsed duration expressed in milliseconds (e.g. `"14ms"`).
//!
//! # Example
//!
//! ```rust,ignore
//! use realestate_api_core::middleware::response_time::response_time_layer;
//! use axum::Router;
//!
//! let app: Router = Router::new()
//!     // ... routes ...
//!     .layer(response_time_layer());
//! ```

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Instant;

use axum::response::Response;

/// HTTP response header name for the elapsed request time.
const RESPONSE_TIME_HEADER: &str = "x-response-time";
use http::Request;
use pin_project_lite::pin_project;
use tower::{Layer, Service};

/// [`tower::Layer`] that wraps each service with [`ResponseTimeService`].
#[derive(Clone)]
pub struct ResponseTimeLayer;

impl<S> Layer<S> for ResponseTimeLayer {
    type Service = ResponseTimeService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ResponseTimeService { inner }
    }
}

/// [`tower::Service`] that records start time and appends `X-Response-Time`
/// to the response once the inner future resolves.
#[derive(Clone)]
pub struct ResponseTimeService<S> {
    inner: S,
}

impl<S, ReqBody> Service<Request<ReqBody>> for ResponseTimeService<S>
where
    S: Service<Request<ReqBody>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = ResponseTimeFuture<S::Future>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let start = Instant::now();
        let fut = self.inner.call(req);
        ResponseTimeFuture { inner: fut, start }
    }
}

pin_project! {
    /// Future returned by [`ResponseTimeService`].
    ///
    /// Polls the inner future and, upon completion, appends the
    /// `X-Response-Time` header to the successful response.
    pub struct ResponseTimeFuture<F> {
        #[pin]
        inner: F,
        start: Instant,
    }
}

impl<F, E> Future for ResponseTimeFuture<F>
where
    F: Future<Output = Result<Response, E>>,
{
    type Output = Result<Response, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        match this.inner.poll(cx) {
            Poll::Ready(Ok(mut response)) => {
                let ms = this.start.elapsed().as_millis();
                let header_val = format!("{ms}ms");
                // The formatted value is always a valid HTTP header value.
                if let Ok(val) = header_val.parse() {
                    response.headers_mut().insert(RESPONSE_TIME_HEADER, val);
                }
                Poll::Ready(Ok(response))
            }
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Creates a [`tower`] layer that appends the `X-Response-Time` header to
/// every response.
pub fn response_time_layer() -> ResponseTimeLayer {
    ResponseTimeLayer
}

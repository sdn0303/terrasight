//! IP-based rate limiting middleware.
//!
//! Wraps [`tower_governor`] (v0.8) to provide a simple constructor for
//! per-IP rate limiting.  The layer uses the peer IP address of the
//! connecting TCP socket as the rate-limit key; place a reverse proxy or
//! `X-Forwarded-For`-aware extractor in front if you need to key on the
//! real client IP behind a load balancer.
//!
//! # Example
//!
//! ```rust,ignore
//! use terrasight_server::http::middleware::rate_limit::{rate_limit_layer, RateLimitConfig};
//! use axum::Router;
//!
//! let layer = rate_limit_layer(&RateLimitConfig {
//!     requests_per_minute: 60,
//!     burst_size: 10,
//! });
//!
//! let app: Router = Router::new()
//!     // ... routes ...
//!     .layer(layer);
//! ```
//!
//! # Note on axum `with_state` ordering
//!
//! The rate-limit layer must be placed **after** all `with_state` calls in
//! the router chain because `GovernorLayer` requires the request to carry a
//! peer address, which Axum attaches during `axum::serve`.

use std::sync::Arc;

use axum::body::Body;

/// Seconds in one minute — used to convert requests-per-minute to a token refill rate.
const SECONDS_PER_MINUTE: u64 = 60;
/// Minimum token refill interval and burst size, preventing division-by-zero.
const MIN_TOKEN_VALUE: u64 = 1;
use governor::{clock::QuantaInstant, middleware::NoOpMiddleware};
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::PeerIpKeyExtractor,
};

/// Concrete type alias for the governor layer produced by this module.
///
/// Uses peer-IP extraction, the default no-op rate-limit middleware, and
/// Axum's [`Body`] as the response body type.
pub type DefaultGovernorLayer =
    GovernorLayer<PeerIpKeyExtractor, NoOpMiddleware<QuantaInstant>, Body>;

/// Configuration for the IP-based rate limiter.
pub struct RateLimitConfig {
    /// Maximum sustained request rate per IP, expressed as requests per minute.
    ///
    /// The underlying token-bucket refill rate is calculated as
    /// `60 / requests_per_minute` seconds per token.  A value of `0` is
    /// clamped to `1` (1 request per 60 seconds).
    pub requests_per_minute: u64,
    /// Instantaneous burst capacity per IP (number of tokens that can
    /// accumulate above the sustained rate).
    ///
    /// A value of `0` is clamped to `1`.
    pub burst_size: u32,
}

/// Creates an IP-based rate-limiting [`DefaultGovernorLayer`] from the
/// supplied [`RateLimitConfig`].
///
/// The token-bucket quota is derived as:
/// - Refill rate: one token every `60 / requests_per_minute` seconds.
/// - Burst capacity: `burst_size` tokens.
///
/// # Panics
///
/// Panics if the internal governor configuration is invalid.  In practice
/// this cannot happen because both `per_second` and `burst_size` are clamped
/// to at least `1` before being passed to [`GovernorConfigBuilder`].
pub fn rate_limit_layer(config: &RateLimitConfig) -> DefaultGovernorLayer {
    let per_second = SECONDS_PER_MINUTE
        .checked_div(config.requests_per_minute.max(MIN_TOKEN_VALUE))
        .unwrap_or(MIN_TOKEN_VALUE)
        .max(MIN_TOKEN_VALUE);
    let burst_size = config.burst_size.max(MIN_TOKEN_VALUE as u32);

    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(per_second)
            .burst_size(burst_size)
            .finish()
            .expect("INVARIANT: GovernorConfig is valid — burst_size >= 1 and period > 0"),
    );

    GovernorLayer::new(governor_config)
}

//! Unified error type for all API clients in this crate.
//!
//! Every fallible public function returns `Result<T, `[`MlitError`]`>`.
//! Callers should match on the variant to decide retry or escalation strategy:
//!
//! - [`MlitError::RateLimited`] — honour the `retry_after_secs` hint.
//! - [`MlitError::Api`] — inspect `status` to distinguish 4xx (caller error)
//!   from 5xx (server error).
//! - [`MlitError::Config`] — fatal; fix deployment configuration.

/// Unified error type for all MLIT API operations.
#[derive(Debug, thiserror::Error)]
pub enum MlitError {
    /// Low-level HTTP transport failure: connection refused, DNS resolution
    /// error, TLS handshake failure, or request timeout.
    ///
    /// Wraps the underlying [`reqwest::Error`] for further inspection.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// The remote API returned a non-2xx HTTP status code (e.g., 400, 401,
    /// 403, 404, 500).
    ///
    /// `status` is the raw HTTP status code. `message` contains the response
    /// body text, which many MLIT endpoints populate with a human-readable
    /// error description.
    #[error("API error {status}: {message}")]
    Api {
        /// Raw HTTP status code (e.g., 401, 403, 500).
        status: u16,
        /// Response body text (often a human-readable error from the MLIT API).
        message: String,
    },

    /// The response body was received successfully but could not be
    /// deserialised into the expected type.
    ///
    /// Occurs when the API changes its response schema or returns an
    /// unexpected content type (e.g., HTML error page instead of JSON).
    #[error("Response parse error: {0}")]
    Parse(String),

    /// The API returned HTTP 429 and all retry attempts were exhausted.
    ///
    /// `retry_after_secs` is the exponential-backoff delay used on the final
    /// retry attempt. Callers should wait at least this many seconds before
    /// retrying at a higher level.
    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited {
        /// Exponential-backoff delay (seconds) used on the final retry attempt.
        retry_after_secs: u64,
    },

    /// A required configuration value (API key or application ID) was absent
    /// from [`crate::config::MlitConfig`].
    ///
    /// The inner `String` names the missing field (e.g.,
    /// `"REINFOLIB_API_KEY is required"`). This is a fatal misconfiguration
    /// error — retrying will not help.
    #[error("Missing configuration: {0}")]
    Config(String),
}

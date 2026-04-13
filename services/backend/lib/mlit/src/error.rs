/// Unified error type for all MLIT API operations.
#[derive(Debug, thiserror::Error)]
pub enum MlitError {
    /// HTTP transport error (connection, timeout, DNS).
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned a non-success status code.
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    /// Response body could not be parsed.
    #[error("Response parse error: {0}")]
    Parse(String),

    /// API rate limit exceeded.
    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    /// Missing required configuration value.
    #[error("Missing configuration: {0}")]
    Config(String),
}

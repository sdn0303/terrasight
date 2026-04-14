//! Crate-wide error hierarchy for the `terrasight-api` domain layer.
//!
//! [`DomainError`] is the single error type that propagates through the
//! `domain` and `usecase` layers. The `handler` layer is responsible for
//! mapping each variant to an appropriate HTTP status code via `AppError`.
//!
//! ## Layer contract
//!
//! - Domain / usecase: return `Result<T, DomainError>`.
//! - Infra: convert `sqlx::Error` → `DomainError::Database` at the
//!   repository boundary so no framework error leaks upward.
//! - Handler: convert `DomainError` → `AppError` (HTTP 4xx / 5xx) once,
//!   at the handler boundary.

/// Framework-independent domain error.
///
/// HTTP status code mapping is the handler layer's responsibility.
/// This type intentionally avoids depending on `axum`, `sqlx`, or any I/O framework.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    /// A geographic coordinate (latitude or longitude) was out of the WGS-84 range.
    ///
    /// Produced by [`BBox::new`](crate::domain::model::BBox::new) and
    /// [`Coord::new`](crate::domain::model::Coord::new). Maps to HTTP 400.
    #[error("Invalid coordinate: {0}")]
    InvalidCoordinate(String),

    /// The requested bounding box exceeds the server-enforced maximum side length.
    ///
    /// Clients must zoom in before requesting spatial data. Maps to HTTP 400.
    #[error("Bounding box exceeds maximum allowed area ({max_deg} degrees per side)")]
    BBoxTooLarge {
        /// The maximum permitted side length in degrees.
        max_deg: f64,
    },

    /// A year value was outside the valid land-price data range.
    ///
    /// Produced by [`Year::new`](crate::domain::model::Year::new).
    /// Maps to HTTP 400.
    #[error("Invalid year: {0}")]
    InvalidYear(String),

    /// A required query or path parameter was absent from the request.
    ///
    /// Produced by handler extractors when a mandatory parameter is missing.
    /// Maps to HTTP 400.
    #[error("Required parameter missing: {0}")]
    MissingParameter(String),

    /// The requested resource does not exist in the database.
    ///
    /// Produced by repository implementations when a `SELECT` returns zero rows
    /// for a single-entity lookup. Maps to HTTP 404.
    #[error("Resource not found")]
    NotFound,

    /// A database operation failed.
    ///
    /// Produced by infra repositories when `sqlx` returns an error. The inner
    /// string contains the `sqlx::Error` display message. Maps to HTTP 503.
    #[error("Database error: {0}")]
    Database(String),

    /// A generic domain validation rule was violated.
    ///
    /// Used for business-rule rejections that do not have a more specific
    /// variant (e.g. invalid enum string, out-of-range numeric parameter).
    /// Maps to HTTP 400.
    #[error("Invalid value: {0}")]
    Validation(String),

    /// An operation exceeded its allowed time budget.
    ///
    /// Produced by the opportunities usecase when the end-to-end TLS
    /// enrichment exceeds `OPPORTUNITY_TIMEOUT_SECS`.
    /// Maps to HTTP 408.
    #[error("Request timed out: {0}")]
    Timeout(String),

    /// A prefecture code string was not a valid 2-digit code in `01`–`47`.
    ///
    /// Produced by [`PrefCode::new`](crate::domain::model::PrefCode::new).
    /// Maps to HTTP 400.
    #[error("invalid prefecture code: {0}")]
    InvalidPrefCode(String),

    /// A municipality code string was not a valid 5-digit JIS X 0402 code.
    ///
    /// Produced by [`CityCode::new`](crate::domain::model::CityCode::new).
    /// Maps to HTTP 400.
    #[error("invalid city code: {0}")]
    InvalidCityCode(String),
}

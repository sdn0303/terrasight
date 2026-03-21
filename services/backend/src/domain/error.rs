/// Framework-independent domain error.
///
/// HTTP status code mapping is the handler layer's responsibility.
/// This type intentionally avoids depending on `axum`, `sqlx`, or any I/O framework.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid coordinate: {0}")]
    InvalidCoordinate(String),

    #[error("Bounding box exceeds maximum allowed area (0.5 degrees per side)")]
    BBoxTooLarge,

    #[error("Required parameter missing: {0}")]
    MissingParameter(String),

    #[error("Resource not found")]
    NotFound,

    #[error("Database error: {0}")]
    Database(String),
}

use axum::http::StatusCode;
use realestate_api_core::error::{ApiError, ErrorMapping};

use crate::domain::error::DomainError;

/// Implement `ErrorMapping` for `DomainError` so `ApiError<DomainError>`
/// produces the correct HTTP status and machine-readable error code.
impl ErrorMapping for DomainError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::InvalidCoordinate(_) => StatusCode::BAD_REQUEST,
            Self::BBoxTooLarge => StatusCode::BAD_REQUEST,
            Self::InvalidYear(_) => StatusCode::BAD_REQUEST,
            Self::MissingParameter(_) => StatusCode::BAD_REQUEST,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::InvalidPrefCode(_) => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Timeout(_) => StatusCode::REQUEST_TIMEOUT,
            Self::Database(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidCoordinate(_) => "INVALID_PARAMS",
            Self::BBoxTooLarge => "BBOX_TOO_LARGE",
            Self::InvalidYear(_) => "INVALID_PARAMS",
            Self::MissingParameter(_) => "INVALID_PARAMS",
            Self::Validation(_) => "INVALID_PARAMS",
            Self::InvalidPrefCode(_) => "INVALID_PARAMS",
            Self::NotFound => "NOT_FOUND",
            Self::Timeout(_) => "TIMEOUT",
            Self::Database(_) => "DB_UNAVAILABLE",
        }
    }
}

/// Application-level HTTP error type.
///
/// A thin alias over [`ApiError<DomainError>`] — no behaviour lives here;
/// all status/code mapping is in the [`ErrorMapping`] impl above.
pub type AppError = ApiError<DomainError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_coordinate_maps_to_400() {
        let err: AppError = DomainError::InvalidCoordinate("bad lat".into()).into();
        assert_eq!(err.0.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.0.error_code(), "INVALID_PARAMS");
    }

    #[test]
    fn not_found_maps_to_404() {
        let err: AppError = DomainError::NotFound.into();
        assert_eq!(err.0.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(err.0.error_code(), "NOT_FOUND");
    }

    #[test]
    fn database_error_maps_to_503() {
        let err: AppError = DomainError::Database("connection refused".into()).into();
        assert_eq!(err.0.status_code(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(err.0.error_code(), "DB_UNAVAILABLE");
    }

    #[test]
    fn validation_maps_to_400() {
        let err: AppError = DomainError::Validation("name must be non-empty".into()).into();
        assert_eq!(err.0.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.0.error_code(), "INVALID_PARAMS");
    }

    #[test]
    fn timeout_maps_to_408() {
        let err: AppError = DomainError::Timeout("opportunities query".into()).into();
        assert_eq!(err.0.status_code(), StatusCode::REQUEST_TIMEOUT);
        assert_eq!(err.0.error_code(), "TIMEOUT");
    }
}

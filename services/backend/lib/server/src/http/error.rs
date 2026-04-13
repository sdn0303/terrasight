use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::fmt;

/// Trait for mapping application-specific errors to HTTP responses.
///
/// Implement this for your domain error type to get automatic
/// `IntoResponse` via [`ApiError<E>`].
///
/// # Example
///
/// ```rust
/// use axum::http::StatusCode;
/// use terrasight_server::http::error::ErrorMapping;
/// use std::fmt;
///
/// #[derive(Debug)]
/// enum MyError {
///     NotFound,
///     Invalid(String),
/// }
///
/// impl fmt::Display for MyError {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         match self {
///             Self::NotFound => write!(f, "Not found"),
///             Self::Invalid(msg) => write!(f, "Invalid: {msg}"),
///         }
///     }
/// }
///
/// impl ErrorMapping for MyError {
///     fn status_code(&self) -> StatusCode {
///         match self {
///             Self::NotFound => StatusCode::NOT_FOUND,
///             Self::Invalid(_) => StatusCode::BAD_REQUEST,
///         }
///     }
///     fn error_code(&self) -> &'static str {
///         match self {
///             Self::NotFound => "NOT_FOUND",
///             Self::Invalid(_) => "INVALID_PARAMS",
///         }
///     }
/// }
/// ```
pub trait ErrorMapping: fmt::Display + fmt::Debug + Send + Sync + 'static {
    /// HTTP status code for this error variant.
    fn status_code(&self) -> StatusCode;

    /// Machine-readable error code (e.g., `"INVALID_PARAMS"`, `"NOT_FOUND"`).
    fn error_code(&self) -> &'static str;
}

/// Generic HTTP error wrapper.
///
/// Wraps any error implementing [`ErrorMapping`] and converts it to a
/// consistent JSON response:
///
/// ```json
/// { "error": { "code": "ERROR_CODE", "message": "Human-readable message" } }
/// ```
///
/// Logs a `WARN` trace event on every conversion to aid observability.
#[derive(Debug)]
pub struct ApiError<E: ErrorMapping>(pub E);

impl<E: ErrorMapping> From<E> for ApiError<E> {
    fn from(e: E) -> Self {
        Self(e)
    }
}

impl<E: ErrorMapping> IntoResponse for ApiError<E> {
    fn into_response(self) -> Response {
        let status = self.0.status_code();
        let body = json!({
            "error": {
                "code": self.0.error_code(),
                "message": self.0.to_string(),
            }
        });

        tracing::warn!(
            status = status.as_u16(),
            error_code = self.0.error_code(),
            error_message = %self.0,
            "API error response"
        );

        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    enum TestError {
        BadInput(String),
        NotFound,
        Internal(String),
    }

    impl fmt::Display for TestError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::BadInput(msg) => write!(f, "Bad input: {msg}"),
                Self::NotFound => write!(f, "Resource not found"),
                Self::Internal(msg) => write!(f, "Internal error: {msg}"),
            }
        }
    }

    impl ErrorMapping for TestError {
        fn status_code(&self) -> StatusCode {
            match self {
                Self::BadInput(_) => StatusCode::BAD_REQUEST,
                Self::NotFound => StatusCode::NOT_FOUND,
                Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }

        fn error_code(&self) -> &'static str {
            match self {
                Self::BadInput(_) => "BAD_INPUT",
                Self::NotFound => "NOT_FOUND",
                Self::Internal(_) => "INTERNAL_ERROR",
            }
        }
    }

    #[test]
    fn api_error_from_test_error() {
        let err = ApiError::from(TestError::BadInput("bad lat".into()));
        assert_eq!(err.0.status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(err.0.error_code(), "BAD_INPUT");
    }

    #[test]
    fn api_error_not_found() {
        let err = ApiError::from(TestError::NotFound);
        assert_eq!(err.0.status_code(), StatusCode::NOT_FOUND);
        assert_eq!(err.0.error_code(), "NOT_FOUND");
    }

    #[test]
    fn api_error_into_response_returns_correct_status() {
        let err = ApiError::from(TestError::Internal("db down".into()));
        let response = err.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}

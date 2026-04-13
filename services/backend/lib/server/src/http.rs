/// Generic API error handling utilities.
pub mod error;

/// Tower middleware: rate limiting and response time.
pub mod middleware;

/// GeoJSON response DTO types.
pub mod response;

/// HTTP tracing layer and request ID propagation.
pub mod tracing;

pub use error::{ApiError, ErrorMapping};
pub use response::{FeatureCollectionDto, FeatureDto};

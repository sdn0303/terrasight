//! Shared helper for converting `sqlx::Error` into the application's
//! [`DomainError`].
//!
//! Extracted from `infra.rs` so every repository impl imports from a
//! dedicated module rather than from the infra parent module.

use crate::domain::error::DomainError;

/// Convert a sqlx database error into a [`DomainError::Database`] with a
/// redacted message produced by `terrasight_server::db::error::map_db_err`.
///
/// Emits a single `error`-level log line with the original error string so
/// the redacted client message stays scannable in production logs.
pub(crate) fn map_db_err(e: sqlx::Error) -> DomainError {
    tracing::error!(error = %e, "database query failed");
    DomainError::Database(terrasight_server::db::error::map_db_err(e).into_message())
}

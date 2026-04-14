//! Shared helper for converting `sqlx::Error` into the application's
//! [`DomainError`].
//!
//! Extracted from `infra.rs` so every repository impl imports from a
//! dedicated module rather than from the infra parent module.

use crate::domain::error::DomainError;

/// Convert a sqlx database error into a [`DomainError::Database`] with a
/// generic client-safe message.
///
/// The original `sqlx::Error` is logged at `error` level with full details
/// (table names, constraint names, connection strings) but **never** forwarded
/// to the HTTP response. Clients receive only `"database error"`.
pub(crate) fn map_db_err(e: sqlx::Error) -> DomainError {
    tracing::error!(error = %e, "database query failed");
    DomainError::Database("database error".into())
}

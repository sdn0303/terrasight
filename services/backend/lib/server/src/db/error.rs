//! Database error type and convenience adapter for `sqlx::Error`.
//!
//! [`DbError`] is a thin newtype over [`sqlx::Error`] that keeps the `db`
//! module's public API independent of `sqlx` internals.  Repository
//! implementations convert `sqlx::Error` with [`map_db_err`] and callers
//! (typically usecase or handler layers) convert the result further into their
//! own domain error:
//!
//! ```rust,ignore
//! use terrasight_server::db::{DbError, map_db_err};
//!
//! let rows = sqlx::query_as::<_, MyRow>(SQL)
//!     .fetch_all(&pool)
//!     .await
//!     .map_err(map_db_err)?;
//! // In the usecase layer:
//! // .map_err(|e: DbError| DomainError::Database(e.into_message()))?;
//! ```

/// Database error type that wraps `sqlx::Error`.
///
/// Domain-independent: the API binary converts this to its own `DomainError`
/// via `.map_err(|e| DomainError::Database(e.to_string()))`.
#[derive(Debug, thiserror::Error)]
#[error("Database error: {0}")]
pub struct DbError(#[from] sqlx::Error);

impl DbError {
    /// Convert to a human-readable string for domain error construction.
    pub fn into_message(self) -> String {
        self.0.to_string()
    }
}

/// Convenience function to map `sqlx::Error` into [`DbError`].
///
/// Use with `.map_err(map_db_err)` in repository implementations.
pub fn map_db_err(e: sqlx::Error) -> DbError {
    DbError(e)
}

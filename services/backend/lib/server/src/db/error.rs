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

pub mod pg_area_repository;
pub mod pg_health_repository;
pub mod pg_score_repository;
pub mod pg_stats_repository;
pub mod pg_trend_repository;
pub mod reinfolib_mock;

/// Convert a database error to a domain error.
///
/// Bridges [`realestate_db::error::DbError`] to the application's [`crate::domain::error::DomainError`].
/// All infra repository implementations use this instead of defining their own local `map_db_err`.
pub(crate) fn map_db_err(e: sqlx::Error) -> crate::domain::error::DomainError {
    tracing::error!(error = %e, "database query failed");
    crate::domain::error::DomainError::Database(realestate_db::error::map_db_err(e).into_message())
}

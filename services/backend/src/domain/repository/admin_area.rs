//! [`AdminAreaStatsRepository`] trait — aggregated statistics for prefecture and municipality areas.

use async_trait::async_trait;

use crate::domain::entity::AdminAreaStats;
use crate::domain::error::DomainError;
use crate::domain::value_object::AreaCode;

/// Repository for administrative area aggregate statistics.
///
/// Accepts both prefecture (2-digit) and municipality (5-digit) area codes.
///
/// Implemented by `PgAdminAreaStatsRepository` in the `infra` layer.
#[async_trait]
pub trait AdminAreaStatsRepository: Send + Sync {
    /// Fetch aggregated statistics for the given administrative area code.
    ///
    /// `code` is a prefecture code (e.g. `"13"`) or municipality code (e.g. `"13105"`).
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn get_area_stats(&self, code: &AreaCode) -> Result<AdminAreaStats, DomainError>;
}

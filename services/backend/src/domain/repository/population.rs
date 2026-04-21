//! [`PopulationRepository`] trait — municipality-level population data from e-Stat.

use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::model::{PopulationSummary, PrefCode};

/// Repository for municipality-level population summaries.
///
/// Queries the `mv_population_summary` materialized view joined with
/// `admin_boundaries` for city names.
///
/// Implemented by `PgPopulationRepository` in the `infra` layer.
#[async_trait]
pub trait PopulationRepository: Send + Sync {
    /// Fetch population summaries for all municipalities in the given prefecture.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_by_pref_code(
        &self,
        pref_code: &PrefCode,
    ) -> Result<Vec<PopulationSummary>, DomainError>;
}

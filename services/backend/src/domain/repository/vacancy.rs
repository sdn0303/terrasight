//! [`VacancyRepository`] trait — municipality-level vacancy data from e-Stat.

use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::model::{PrefCode, VacancySummary};

/// Repository for municipality-level housing vacancy summaries.
///
/// Queries the `mv_vacancy_summary` materialized view joined with
/// `admin_boundaries` for city names.
///
/// Implemented by `PgVacancyRepository` in the `infra` layer.
#[async_trait]
pub trait VacancyRepository: Send + Sync {
    /// Fetch vacancy summaries for all municipalities in the given prefecture.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_by_pref_code(
        &self,
        pref_code: &PrefCode,
    ) -> Result<Vec<VacancySummary>, DomainError>;
}

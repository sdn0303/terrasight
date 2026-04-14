//! [`AppraisalRepository`] trait — official land appraisal (鑑定評価) records from MLIT reinfolib.

use async_trait::async_trait;

use crate::domain::appraisal::AppraisalDetail;
use crate::domain::error::DomainError;
use crate::domain::value_object::{CityCode, PrefCode};

/// Repository for official land appraisal (鑑定評価) records.
///
/// Queries the `appraisals` table sourced from MLIT reinfolib.
///
/// Implemented by `PgAppraisalRepository` in the `infra` layer.
#[async_trait]
pub trait AppraisalRepository: Send + Sync {
    /// Fetch appraisal records for a prefecture, optionally filtered by city code.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_appraisals(
        &self,
        pref_code: &PrefCode,
        city_code: Option<&CityCode>,
    ) -> Result<Vec<AppraisalDetail>, DomainError>;
}

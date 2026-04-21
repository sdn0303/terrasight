//! Usecase: fetch vacancy summaries for a prefecture.
//!
//! Delegates to [`VacancyRepository::find_by_pref_code`] and returns a list
//! of [`VacancySummary`] records. Called by `GET /api/v1/vacancy`.

use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::model::{PrefCode, VacancySummary};
use crate::domain::repository::VacancyRepository;

/// Usecase for `GET /api/v1/vacancy`.
pub struct GetVacancyUsecase {
    vacancy_repo: Arc<dyn VacancyRepository>,
}

impl GetVacancyUsecase {
    /// Construct the usecase with the given vacancy repository.
    pub fn new(vacancy_repo: Arc<dyn VacancyRepository>) -> Self {
        Self { vacancy_repo }
    }

    /// Fetch vacancy summaries for the given prefecture.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    #[tracing::instrument(skip(self), fields(usecase = "get_vacancy"))]
    pub async fn execute(&self, pref_code: &PrefCode) -> Result<Vec<VacancySummary>, DomainError> {
        self.vacancy_repo
            .find_by_pref_code(pref_code)
            .await
            .inspect(|items| {
                tracing::debug!(
                    count = items.len(),
                    pref_code = pref_code.as_str(),
                    "vacancy fetched"
                )
            })
            .inspect_err(|e| tracing::warn!(error = %e, "vacancy lookup failed"))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::*;
    use crate::domain::error::DomainError;
    use crate::domain::model::{AreaName, CityCode, PrefCode, VacancySummary};

    fn sample_summary() -> VacancySummary {
        VacancySummary {
            city_code: CityCode::new("13104").unwrap(),
            city_name: AreaName::parse("新宿区").unwrap(),
            vacancy_count: 12_500,
            total_houses: Some(210_000),
            vacancy_rate_pct: Some(5.95),
            survey_year: 2023,
        }
    }

    fn pref() -> PrefCode {
        PrefCode::new("13").unwrap()
    }

    struct OkRepo;

    #[async_trait]
    impl VacancyRepository for OkRepo {
        async fn find_by_pref_code(
            &self,
            _pref_code: &PrefCode,
        ) -> Result<Vec<VacancySummary>, DomainError> {
            Ok(vec![sample_summary()])
        }
    }

    struct ErrRepo;

    #[async_trait]
    impl VacancyRepository for ErrRepo {
        async fn find_by_pref_code(
            &self,
            _pref_code: &PrefCode,
        ) -> Result<Vec<VacancySummary>, DomainError> {
            Err(DomainError::Database("boom".into()))
        }
    }

    #[tokio::test]
    async fn execute_returns_items_from_repo() {
        let usecase = GetVacancyUsecase::new(Arc::new(OkRepo));
        let result = usecase.execute(&pref()).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].city_code.as_str(), "13104");
        assert_eq!(result[0].vacancy_count, 12_500);
        assert_eq!(result[0].vacancy_rate_pct, Some(5.95));
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let usecase = GetVacancyUsecase::new(Arc::new(ErrRepo));
        let err = usecase.execute(&pref()).await.unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

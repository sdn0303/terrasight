//! Usecase: fetch population summaries for a prefecture.
//!
//! Delegates to [`PopulationRepository::find_by_pref_code`] and returns a list
//! of [`PopulationSummary`] records. Called by `GET /api/v1/population`.

use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::model::{PopulationSummary, PrefCode};
use crate::domain::repository::PopulationRepository;

/// Usecase for `GET /api/v1/population`.
pub struct GetPopulationUsecase {
    population_repo: Arc<dyn PopulationRepository>,
}

impl GetPopulationUsecase {
    /// Construct the usecase with the given population repository.
    pub fn new(population_repo: Arc<dyn PopulationRepository>) -> Self {
        Self { population_repo }
    }

    /// Fetch population summaries for the given prefecture.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    #[tracing::instrument(skip(self), fields(usecase = "get_population"))]
    pub async fn execute(
        &self,
        pref_code: &PrefCode,
    ) -> Result<Vec<PopulationSummary>, DomainError> {
        self.population_repo
            .find_by_pref_code(pref_code)
            .await
            .inspect(|items| {
                tracing::debug!(
                    count = items.len(),
                    pref_code = pref_code.as_str(),
                    "population fetched"
                )
            })
            .inspect_err(|e| tracing::warn!(error = %e, "population lookup failed"))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::*;
    use crate::domain::error::DomainError;
    use crate::domain::model::{AreaName, CityCode, PopulationSummary, PrefCode};

    fn sample_summary() -> PopulationSummary {
        PopulationSummary {
            city_code: CityCode::new("13104").unwrap(),
            city_name: AreaName::parse("新宿区").unwrap(),
            population: 344_880,
            male: Some(172_000),
            female: Some(172_880),
            households: Some(206_093),
            census_year: 2020,
        }
    }

    fn pref() -> PrefCode {
        PrefCode::new("13").unwrap()
    }

    struct OkRepo;

    #[async_trait]
    impl PopulationRepository for OkRepo {
        async fn find_by_pref_code(
            &self,
            _pref_code: &PrefCode,
        ) -> Result<Vec<PopulationSummary>, DomainError> {
            Ok(vec![sample_summary()])
        }
    }

    struct ErrRepo;

    #[async_trait]
    impl PopulationRepository for ErrRepo {
        async fn find_by_pref_code(
            &self,
            _pref_code: &PrefCode,
        ) -> Result<Vec<PopulationSummary>, DomainError> {
            Err(DomainError::Database("boom".into()))
        }
    }

    #[tokio::test]
    async fn execute_returns_items_from_repo() {
        let usecase = GetPopulationUsecase::new(Arc::new(OkRepo));
        let result = usecase.execute(&pref()).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].city_code.as_str(), "13104");
        assert_eq!(result[0].population, 344_880);
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let usecase = GetPopulationUsecase::new(Arc::new(ErrRepo));
        let err = usecase.execute(&pref()).await.unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

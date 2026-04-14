//! Usecase: fetch municipalities for a prefecture.
//!
//! Delegates to [`MunicipalityRepository::find_municipalities`]. Returns a
//! list of [`Municipality`] records ordered by `city_code`. Called by
//! `GET /api/v1/municipalities`.

use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::municipality::Municipality;
use crate::domain::repository::MunicipalityRepository;
use crate::domain::value_object::PrefCode;

/// Usecase for `GET /api/v1/municipalities`.
pub struct GetMunicipalitiesUsecase {
    municipality_repo: Arc<dyn MunicipalityRepository>,
}

impl GetMunicipalitiesUsecase {
    /// Construct the usecase with the given repository.
    pub fn new(municipality_repo: Arc<dyn MunicipalityRepository>) -> Self {
        Self { municipality_repo }
    }

    /// Fetch all municipalities for the given prefecture.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    #[tracing::instrument(skip(self), fields(usecase = "get_municipalities"))]
    pub async fn execute(&self, pref_code: &PrefCode) -> Result<Vec<Municipality>, DomainError> {
        self.municipality_repo
            .find_municipalities(pref_code)
            .await
            .inspect(|items| {
                tracing::debug!(
                    count = items.len(),
                    pref_code = pref_code.as_str(),
                    "municipalities fetched"
                )
            })
            .inspect_err(|e| tracing::warn!(error = %e, "municipalities lookup failed"))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::*;
    use crate::domain::error::DomainError;

    fn sample_municipality() -> Municipality {
        use crate::domain::entity::AreaName;
        use crate::domain::value_object::{CityCode, PrefCode};
        Municipality {
            city_code: CityCode::new("13101").unwrap(),
            city_name: AreaName::parse("千代田区").unwrap(),
            pref_code: PrefCode::new("13").unwrap(),
        }
    }

    fn pref() -> PrefCode {
        PrefCode::new("13").unwrap()
    }

    // ── happy-path mock ───────────────────────────────────────────────────────

    struct OkRepo;

    #[async_trait]
    impl MunicipalityRepository for OkRepo {
        async fn find_municipalities(
            &self,
            _pref_code: &PrefCode,
        ) -> Result<Vec<Municipality>, DomainError> {
            Ok(vec![sample_municipality()])
        }
    }

    // ── error mock ────────────────────────────────────────────────────────────

    struct ErrRepo;

    #[async_trait]
    impl MunicipalityRepository for ErrRepo {
        async fn find_municipalities(
            &self,
            _pref_code: &PrefCode,
        ) -> Result<Vec<Municipality>, DomainError> {
            Err(DomainError::Database("boom".into()))
        }
    }

    #[tokio::test]
    async fn execute_returns_municipalities() {
        let usecase = GetMunicipalitiesUsecase::new(Arc::new(OkRepo));
        let result = usecase.execute(&pref()).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].city_code.as_str(), "13101");
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let usecase = GetMunicipalitiesUsecase::new(Arc::new(ErrRepo));
        let err = usecase.execute(&pref()).await.unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

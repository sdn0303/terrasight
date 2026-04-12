use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::municipality::Municipality;
use crate::domain::repository::MunicipalityRepository;
use crate::domain::value_object::PrefCode;

pub struct GetMunicipalitiesUsecase {
    municipality_repo: Arc<dyn MunicipalityRepository>,
}

impl GetMunicipalitiesUsecase {
    pub fn new(municipality_repo: Arc<dyn MunicipalityRepository>) -> Self {
        Self { municipality_repo }
    }

    /// Fetch all municipalities for the given prefecture.
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
        Municipality {
            city_code: "13101".into(),
            city_name: "千代田区".into(),
            pref_code: "13".into(),
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
        assert_eq!(result[0].city_code, "13101");
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let usecase = GetMunicipalitiesUsecase::new(Arc::new(ErrRepo));
        let err = usecase.execute(&pref()).await.unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

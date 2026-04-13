//! Usecase: fetch MLIT appraisal records for a prefecture.
//!
//! Delegates to [`AppraisalRepository::find_appraisals`] and returns a list
//! of [`AppraisalDetail`] records optionally narrowed to a single municipality.
//! Called by `GET /api/v1/appraisals`.

use std::sync::Arc;

use crate::domain::appraisal::AppraisalDetail;
use crate::domain::error::DomainError;
use crate::domain::repository::AppraisalRepository;
use crate::domain::value_object::PrefCode;

/// Usecase for `GET /api/v1/appraisals`.
pub struct GetAppraisalsUsecase {
    appraisal_repo: Arc<dyn AppraisalRepository>,
}

impl GetAppraisalsUsecase {
    /// Construct the usecase with the given appraisal repository.
    pub fn new(appraisal_repo: Arc<dyn AppraisalRepository>) -> Self {
        Self { appraisal_repo }
    }

    /// Fetch appraisal records for the given prefecture, optionally filtered by city code.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    #[tracing::instrument(skip(self), fields(usecase = "get_appraisals"))]
    pub async fn execute(
        &self,
        pref_code: &PrefCode,
        city_code: Option<&str>,
    ) -> Result<Vec<AppraisalDetail>, DomainError> {
        self.appraisal_repo
            .find_appraisals(pref_code, city_code)
            .await
            .inspect(|items| {
                tracing::debug!(
                    count = items.len(),
                    pref_code = pref_code.as_str(),
                    "appraisals fetched"
                )
            })
            .inspect_err(|e| tracing::warn!(error = %e, "appraisals lookup failed"))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use super::*;
    use crate::domain::error::DomainError;

    fn sample_detail() -> AppraisalDetail {
        AppraisalDetail {
            city_code: "13101".into(),
            city_name: "千代田区".into(),
            address: "千代田1-1".into(),
            land_use_code: "01".into(),
            price_per_sqm: 1_000_000,
            appraisal_price: 50_000_000,
            lot_area_sqm: Some(50.0),
            zone_code: None,
            building_coverage: None,
            floor_area_ratio: None,
            comparable_price: None,
            yield_price: None,
            cost_price: None,
            fudosan_id: None,
        }
    }

    fn pref() -> PrefCode {
        PrefCode::new("13").unwrap()
    }

    // ── happy-path mock ───────────────────────────────────────────────────────

    struct OkRepo;

    #[async_trait]
    impl AppraisalRepository for OkRepo {
        async fn find_appraisals(
            &self,
            _pref_code: &PrefCode,
            _city_code: Option<&str>,
        ) -> Result<Vec<AppraisalDetail>, DomainError> {
            Ok(vec![sample_detail()])
        }
    }

    // ── error mock ────────────────────────────────────────────────────────────

    struct ErrRepo;

    #[async_trait]
    impl AppraisalRepository for ErrRepo {
        async fn find_appraisals(
            &self,
            _pref_code: &PrefCode,
            _city_code: Option<&str>,
        ) -> Result<Vec<AppraisalDetail>, DomainError> {
            Err(DomainError::Database("boom".into()))
        }
    }

    #[tokio::test]
    async fn execute_returns_items_from_repo() {
        let usecase = GetAppraisalsUsecase::new(Arc::new(OkRepo));
        let result = usecase.execute(&pref(), None).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].city_code, "13101");
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let usecase = GetAppraisalsUsecase::new(Arc::new(ErrRepo));
        let err = usecase.execute(&pref(), None).await.unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

//! Usecase: fetch aggregated transaction summaries for a prefecture.
//!
//! Delegates to [`TransactionRepository::find_transaction_summary`] and
//! returns a list of [`TransactionSummary`] records. Called by
//! `GET /api/v1/transactions/summary`.

use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::repository::TransactionRepository;
use crate::domain::transaction::TransactionSummary;
use crate::domain::value_object::{PrefCode, Year};

/// Usecase for `GET /api/v1/transactions/summary`.
pub struct GetTransactionSummaryUsecase {
    repo: Arc<dyn TransactionRepository>,
}

impl GetTransactionSummaryUsecase {
    /// Construct the usecase with the given repository.
    pub fn new(repo: Arc<dyn TransactionRepository>) -> Self {
        Self { repo }
    }

    /// Fetch aggregated transaction summaries for the given prefecture.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    #[tracing::instrument(skip(self), fields(usecase = "get_transaction_summary"))]
    pub async fn execute(
        &self,
        pref_code: &PrefCode,
        year_from: Option<&Year>,
        property_type: Option<&str>,
    ) -> Result<Vec<TransactionSummary>, DomainError> {
        self.repo
            .find_transaction_summary(pref_code, year_from, property_type)
            .await
            .inspect(|v| tracing::debug!(count = v.len(), "transaction summary fetched"))
            .inspect_err(|e| tracing::warn!(error = %e, "transaction summary failed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::mock::MockTransactionRepository;

    fn sample_summary() -> TransactionSummary {
        TransactionSummary {
            city_code: "13101".into(),
            transaction_year: 2023,
            property_type: "宅地(土地)".into(),
            tx_count: 42,
            avg_total_price: 50_000_000,
            median_total_price: 45_000_000,
            avg_price_sqm: Some(800_000),
            avg_area_sqm: Some(60),
            avg_walk_min: Some(8),
        }
    }

    #[tokio::test]
    async fn execute_returns_summaries() {
        let pref = PrefCode::new("13").unwrap();
        let repo =
            Arc::new(MockTransactionRepository::new().with_summary(Ok(vec![sample_summary()])));
        let usecase = GetTransactionSummaryUsecase::new(repo);
        let result = usecase.execute(&pref, None, None).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].city_code, "13101");
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let pref = PrefCode::new("13").unwrap();
        let repo = Arc::new(
            MockTransactionRepository::new()
                .with_summary(Err(DomainError::Database("boom".into()))),
        );
        let usecase = GetTransactionSummaryUsecase::new(repo);
        let err = usecase.execute(&pref, None, None).await.unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

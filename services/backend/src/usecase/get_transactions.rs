use std::sync::Arc;

use crate::domain::constants::{DEFAULT_TRANSACTION_LIMIT, MAX_TRANSACTION_LIMIT};
use crate::domain::error::DomainError;
use crate::domain::repository::TransactionRepository;
use crate::domain::transaction::TransactionDetail;
use crate::domain::value_object::Year;

pub struct GetTransactionsUsecase {
    repo: Arc<dyn TransactionRepository>,
}

impl GetTransactionsUsecase {
    pub fn new(repo: Arc<dyn TransactionRepository>) -> Self {
        Self { repo }
    }

    /// Fetch individual transaction records for the given city code.
    ///
    /// `limit` is clamped to `[1, DEFAULT_TRANSACTION_LIMIT]` when `None`.
    #[tracing::instrument(skip(self), fields(usecase = "get_transactions"))]
    pub async fn execute(
        &self,
        city_code: &str,
        year_from: Option<&Year>,
        limit: Option<u32>,
    ) -> Result<Vec<TransactionDetail>, DomainError> {
        let limit = limit
            .unwrap_or(DEFAULT_TRANSACTION_LIMIT)
            .min(MAX_TRANSACTION_LIMIT);
        self.repo
            .find_transactions(city_code, year_from, limit)
            .await
            .inspect(|v| tracing::debug!(count = v.len(), "transactions fetched"))
            .inspect_err(|e| tracing::warn!(error = %e, "transactions failed"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::mock::MockTransactionRepository;

    fn sample_detail() -> TransactionDetail {
        TransactionDetail {
            city_code: "13101".into(),
            city_name: "千代田区".into(),
            district_name: None,
            property_type: "宅地(土地)".into(),
            total_price: 80_000_000,
            price_per_sqm: Some(1_200_000),
            area_sqm: Some(66),
            floor_plan: None,
            building_year: Some(2010),
            building_structure: Some("RC".into()),
            nearest_station: Some("東京".into()),
            station_walk_min: Some(5),
            transaction_quarter: "2023Q1".into(),
        }
    }

    #[tokio::test]
    async fn execute_returns_details() {
        let repo =
            Arc::new(MockTransactionRepository::new().with_transactions(Ok(vec![sample_detail()])));
        let usecase = GetTransactionsUsecase::new(repo);
        let result = usecase.execute("13101", None, None).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].city_name, "千代田区");
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let repo = Arc::new(
            MockTransactionRepository::new()
                .with_transactions(Err(DomainError::Database("boom".into()))),
        );
        let usecase = GetTransactionsUsecase::new(repo);
        let err = usecase.execute("13101", None, None).await.unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

//! [`TransactionRepository`] trait — real-estate transaction data from MLIT reinfolib XPT003.

use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::transaction::{TransactionDetail, TransactionSummary};
use crate::domain::value_object::{CityCode, PrefCode, Year};

/// Repository for real-estate transaction data.
///
/// Queries the `transactions` table sourced from MLIT reinfolib XPT003.
///
/// Implemented by `PgTransactionRepository` in the `infra` layer.
#[async_trait]
pub trait TransactionRepository: Send + Sync {
    /// Fetch aggregated transaction summaries per city/year/property_type.
    ///
    /// `year_from` optionally restricts results to records on or after that year.
    /// `property_type` optionally restricts to a single property type string.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_transaction_summary(
        &self,
        pref_code: &PrefCode,
        year_from: Option<&Year>,
        property_type: Option<&str>,
    ) -> Result<Vec<TransactionSummary>, DomainError>;

    /// Fetch individual transaction records for a given city code.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_transactions(
        &self,
        city_code: &CityCode,
        year_from: Option<&Year>,
        limit: u32,
    ) -> Result<Vec<TransactionDetail>, DomainError>;
}

//! Request DTOs for the transaction endpoints.
//!
//! `TransactionSummaryQuery` drives `GET /api/v1/transactions/summary`.
//! `TransactionsQuery` drives `GET /api/v1/transactions`.

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::model::{PrefCode, Year};

/// Query parameters for `GET /api/v1/transactions/summary`.
#[derive(Debug, Deserialize)]
pub struct TransactionSummaryQuery {
    pub pref_code: String,
    pub year_from: Option<i32>,
    pub property_type: Option<String>,
}

impl TransactionSummaryQuery {
    /// Convert to validated domain value objects.
    pub fn into_domain(self) -> Result<(PrefCode, Option<Year>, Option<String>), DomainError> {
        let pref = PrefCode::new(&self.pref_code)?;
        let year = self.year_from.map(Year::new).transpose()?;
        Ok((pref, year, self.property_type))
    }
}

/// Query parameters for `GET /api/v1/transactions`.
#[derive(Debug, Deserialize)]
pub struct TransactionsQuery {
    pub city_code: String,
    pub year_from: Option<i32>,
    pub limit: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_query_valid_pref_and_year() {
        let q = TransactionSummaryQuery {
            pref_code: "13".into(),
            year_from: Some(2020),
            property_type: Some("宅地(土地)".into()),
        };
        let (pref, year, prop) = q.into_domain().unwrap();
        assert_eq!(pref.as_str(), "13");
        assert_eq!(year.unwrap().value(), 2020);
        assert_eq!(prop.as_deref(), Some("宅地(土地)"));
    }

    #[test]
    fn summary_query_invalid_pref_code_returns_error() {
        let q = TransactionSummaryQuery {
            pref_code: "00".into(),
            year_from: None,
            property_type: None,
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn summary_query_invalid_year_returns_error() {
        let q = TransactionSummaryQuery {
            pref_code: "13".into(),
            year_from: Some(1999),
            property_type: None,
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn summary_query_optional_fields_omitted() {
        let q = TransactionSummaryQuery {
            pref_code: "13".into(),
            year_from: None,
            property_type: None,
        };
        let (pref, year, prop) = q.into_domain().unwrap();
        assert_eq!(pref.as_str(), "13");
        assert!(year.is_none());
        assert!(prop.is_none());
    }
}

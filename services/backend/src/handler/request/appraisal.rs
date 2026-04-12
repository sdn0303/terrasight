//! Request DTO for `GET /api/v1/appraisals`.

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::value_object::PrefCode;

/// Query parameters for `GET /api/v1/appraisals`.
///
/// `pref_code` is required; `city_code` optionally narrows results to a
/// single municipality.
#[derive(Debug, Deserialize)]
pub struct AppraisalsQuery {
    pub pref_code: String,
    pub city_code: Option<String>,
}

impl AppraisalsQuery {
    /// Convert to domain value objects.
    ///
    /// `pref_code` is validated via [`PrefCode::new`].
    /// `city_code` is passed through as-is (raw 5-digit JIS X 0402 string).
    pub fn into_domain(self) -> Result<(PrefCode, Option<String>), DomainError> {
        let pref = PrefCode::new(&self.pref_code)?;
        Ok((pref, self.city_code))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_domain_valid_pref_no_city() {
        let q = AppraisalsQuery {
            pref_code: "13".into(),
            city_code: None,
        };
        let (pref, city) = q.into_domain().unwrap();
        assert_eq!(pref.as_str(), "13");
        assert!(city.is_none());
    }

    #[test]
    fn into_domain_valid_pref_with_city() {
        let q = AppraisalsQuery {
            pref_code: "13".into(),
            city_code: Some("13101".into()),
        };
        let (pref, city) = q.into_domain().unwrap();
        assert_eq!(pref.as_str(), "13");
        assert_eq!(city.as_deref(), Some("13101"));
    }

    #[test]
    fn into_domain_invalid_pref_returns_err() {
        let q = AppraisalsQuery {
            pref_code: "invalid".into(),
            city_code: None,
        };
        assert!(q.into_domain().is_err());
    }
}

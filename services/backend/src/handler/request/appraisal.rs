//! Request DTO for `GET /api/v1/appraisals`.

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::model::{CityCode, PrefCode};

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
    /// `city_code`, when present, is validated via [`CityCode::new`] and its
    /// prefecture prefix is checked against `pref_code`.
    pub fn into_domain(self) -> Result<(PrefCode, Option<CityCode>), DomainError> {
        let pref = PrefCode::new(&self.pref_code)?;
        let city = self.city_code.as_deref().map(CityCode::new).transpose()?;
        if let Some(ref c) = city
            && c.pref_code() != pref.as_str()
        {
            return Err(DomainError::InvalidCityCode(format!(
                "city_code {} does not belong to pref_code {}",
                c.as_str(),
                pref.as_str(),
            )));
        }
        Ok((pref, city))
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
        assert_eq!(city.as_ref().map(|c| c.as_str()), Some("13101"));
    }

    #[test]
    fn into_domain_invalid_pref_returns_err() {
        let q = AppraisalsQuery {
            pref_code: "invalid".into(),
            city_code: None,
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn into_domain_invalid_city_code_returns_err() {
        let q = AppraisalsQuery {
            pref_code: "13".into(),
            city_code: Some("bad".into()),
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn into_domain_city_code_pref_mismatch_returns_err() {
        let q = AppraisalsQuery {
            pref_code: "13".into(),
            city_code: Some("27102".into()),
        };
        assert!(q.into_domain().is_err());
    }
}

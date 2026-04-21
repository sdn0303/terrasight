//! Request DTO for `GET /api/v1/vacancy`.

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::model::PrefCode;

/// Query parameters for `GET /api/v1/vacancy`.
///
/// `pref_code` is the only required field; it is validated against Japan's
/// valid prefecture code range (`01`–`47`) via [`PrefCode::new`].
#[derive(Debug, Deserialize)]
pub struct VacancyQuery {
    pub pref_code: String,
}

impl VacancyQuery {
    /// Convert raw query string fields to domain value objects.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidPrefCode`] if `pref_code` is not a
    /// valid 2-digit prefecture code.
    pub fn into_domain(self) -> Result<PrefCode, DomainError> {
        PrefCode::new(&self.pref_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_pref_code_parses() {
        let q = VacancyQuery {
            pref_code: "13".into(),
        };
        let pref = q.into_domain().unwrap();
        assert_eq!(pref.as_str(), "13");
    }

    #[test]
    fn invalid_pref_code_returns_err() {
        let q = VacancyQuery {
            pref_code: "00".into(),
        };
        assert!(q.into_domain().is_err());
    }

    #[test]
    fn non_numeric_pref_code_returns_err() {
        let q = VacancyQuery {
            pref_code: "xy".into(),
        };
        assert!(q.into_domain().is_err());
    }
}

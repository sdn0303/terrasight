//! Request DTO for `GET /api/v1/municipalities`.

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::model::PrefCode;

/// Query parameters for `GET /api/v1/municipalities`.
#[derive(Debug, Deserialize)]
pub struct MunicipalitiesQuery {
    pub pref_code: String,
}

impl MunicipalitiesQuery {
    /// Convert to domain value object.
    pub fn into_domain(self) -> Result<PrefCode, DomainError> {
        PrefCode::new(&self.pref_code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_domain_valid() {
        let q = MunicipalitiesQuery {
            pref_code: "13".into(),
        };
        assert_eq!(q.into_domain().unwrap().as_str(), "13");
    }

    #[test]
    fn into_domain_invalid_returns_err() {
        let q = MunicipalitiesQuery {
            pref_code: "abc".into(),
        };
        assert!(q.into_domain().is_err());
    }
}

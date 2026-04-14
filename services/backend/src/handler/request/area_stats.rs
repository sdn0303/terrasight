//! Request DTO for `GET /api/v1/area-stats`.
//!
//! [`AreaStatsQuery`] carries a single `code` field that is validated and
//! parsed into an [`AreaCode`](crate::domain::model::AreaCode) domain
//! value object by [`AreaStatsQuery::into_domain`].

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::model::AreaCode;

/// Query parameters for `GET /api/v1/area-stats`.
#[derive(Debug, Deserialize)]
pub struct AreaStatsQuery {
    /// Administrative area code. Accepts a 2-digit prefecture code
    /// (e.g. `"13"` for Tokyo) or a 5-digit municipality code
    /// (e.g. `"13104"` for Shinjuku-ku).
    pub code: String,
}

impl AreaStatsQuery {
    /// Convert to a validated [`AreaCode`] domain value object.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Validation`] when `code` is not a valid
    /// 2-digit prefecture code or 5-digit municipality code.
    pub fn into_domain(self) -> Result<AreaCode, DomainError> {
        AreaCode::parse(&self.code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_stats_query_valid_prefecture() {
        let q = AreaStatsQuery { code: "13".into() };
        assert!(q.into_domain().is_ok());
    }

    #[test]
    fn area_stats_query_valid_municipality() {
        let q = AreaStatsQuery {
            code: "13104".into(),
        };
        assert!(q.into_domain().is_ok());
    }

    #[test]
    fn area_stats_query_invalid() {
        let q = AreaStatsQuery { code: "abc".into() };
        assert!(q.into_domain().is_err());
    }
}

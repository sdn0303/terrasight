//! Request DTO for `GET /api/area-stats`.

use serde::Deserialize;

use crate::domain::error::DomainError;
use crate::domain::value_object::AreaCode;

#[derive(Debug, Deserialize)]
pub struct AreaStatsQuery {
    pub code: String,
}

impl AreaStatsQuery {
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

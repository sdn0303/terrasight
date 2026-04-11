//! Request DTO for `GET /api/trend`.

use serde::Deserialize;

use crate::domain::constants::TREND_DEFAULT_YEARS;
use crate::domain::error::DomainError;
use crate::domain::value_object::{Coord, YearsLookback};

/// Trend query parameters (includes optional `years`).
#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    pub lat: f64,
    pub lng: f64,
    #[serde(default = "default_years")]
    pub years: i32,
}

fn default_years() -> i32 {
    TREND_DEFAULT_YEARS
}

impl TrendQuery {
    pub fn into_domain(self) -> Result<(Coord, YearsLookback), DomainError> {
        let coord = Coord::new(self.lat, self.lng)?;
        Ok((coord, YearsLookback::clamped(self.years)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trend_query_default_years() {
        let q = TrendQuery {
            lat: 35.68,
            lng: 139.76,
            years: default_years(),
        };
        let (_, years) = q.into_domain().unwrap();
        assert_eq!(years, YearsLookback::clamped(TREND_DEFAULT_YEARS));
    }
}

//! Request DTO for `GET /api/v1/trend`.
//!
//! [`TrendQuery`] carries latitude, longitude, and an optional years
//! lookback window. Converted to `(Coord, YearsLookback)` domain value
//! objects via [`TrendQuery::into_domain`].

use serde::Deserialize;

use crate::domain::constants::TREND_DEFAULT_YEARS;
use crate::domain::error::DomainError;
use crate::domain::model::{Coord, YearsLookback};

/// Query parameters for `GET /api/v1/trend`.
#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    /// Latitude of the point of interest (WGS-84 decimal degrees).
    pub lat: f64,
    /// Longitude of the point of interest (WGS-84 decimal degrees).
    pub lng: f64,
    /// Number of years to look back when computing the CAGR.
    /// Defaults to [`TREND_DEFAULT_YEARS`](crate::domain::constants::TREND_DEFAULT_YEARS);
    /// values outside the valid range are clamped by [`YearsLookback::clamped`].
    #[serde(default = "default_years")]
    pub years: i32,
}

fn default_years() -> i32 {
    TREND_DEFAULT_YEARS
}

impl TrendQuery {
    /// Convert to validated domain value objects.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::InvalidCoordinate`] when `lat` or `lng` is
    /// outside the valid WGS-84 range.
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

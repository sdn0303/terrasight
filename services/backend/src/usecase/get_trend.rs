use std::sync::Arc;

use realestate_geo_math::{finance::compute_cagr, rounding::round_dp};

use crate::domain::constants::PRECISION_RATIO;
use crate::domain::error::DomainError;
use crate::domain::repository::TrendRepository;
use crate::domain::value_object::{Coord, TrendAnalysis, TrendDirection, YearsLookback};

pub struct GetTrendUsecase {
    trend_repo: Arc<dyn TrendRepository>,
}

impl GetTrendUsecase {
    pub fn new(trend_repo: Arc<dyn TrendRepository>) -> Self {
        Self { trend_repo }
    }

    /// Retrieve price trend data for the nearest observation point and compute CAGR.
    ///
    /// Business logic (CAGR calculation, direction determination) lives here
    /// rather than in the handler layer (P1 review fix).
    pub async fn execute(
        &self,
        coord: Coord,
        years: YearsLookback,
    ) -> Result<TrendAnalysis, DomainError> {
        let result = self.trend_repo.find_trend(coord, years).await?;

        let (location, data) = result.ok_or(DomainError::NotFound)?;

        if data.is_empty() {
            tracing::debug!("no trend data found");
            return Err(DomainError::NotFound);
        }

        let distance_m_fmt = format!("{:.1}", location.distance_m); // PRECISION_DISTANCE
        tracing::debug!(
            address = %location.address,
            distance_m = %distance_m_fmt,
            data_points = data.len(),
            "trend data found"
        );

        let first_price = data[0].price_per_sqm as f64;
        let last_price = data[data.len() - 1].price_per_sqm as f64;
        let n_years = (data[data.len() - 1].year - data[0].year).max(1) as u32;

        let cagr = compute_cagr(first_price, last_price, n_years);
        let cagr_rounded = round_dp(cagr, PRECISION_RATIO);
        let direction = if cagr > 0.0 {
            TrendDirection::Up
        } else {
            TrendDirection::Down
        };

        Ok(TrendAnalysis {
            location,
            data,
            cagr: cagr_rounded,
            direction,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::value_object::TrendDirection;

    #[test]
    fn trend_direction_as_str() {
        assert_eq!(TrendDirection::Up.as_str(), "up");
        assert_eq!(TrendDirection::Down.as_str(), "down");
    }
}

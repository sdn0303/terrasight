//! Usecase: compute the land price trend for a coordinate.
//!
//! Fetches the nearest land-price observation series from
//! [`TrendRepository::find_trend`], then computes the Compound Annual Growth
//! Rate (CAGR) and direction in the usecase layer rather than in the handler.
//! Called by `GET /api/v1/trend`.

use std::sync::Arc;

use terrasight_geo::{finance::compute_cagr, rounding::round_dp};

use crate::domain::constants::PRECISION_RATIO;
use crate::domain::error::DomainError;
use crate::domain::repository::TrendRepository;
use crate::domain::value_object::{Coord, TrendAnalysis, TrendDirection, YearsLookback};

/// Usecase for `GET /api/v1/trend`.
pub(crate) struct GetTrendUsecase {
    trend_repo: Arc<dyn TrendRepository>,
}

impl GetTrendUsecase {
    /// Construct the usecase with the given trend repository.
    pub(crate) fn new(trend_repo: Arc<dyn TrendRepository>) -> Self {
        Self { trend_repo }
    }

    /// Retrieve the price trend for the nearest observation point and compute CAGR.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::NotFound`] when no observation point is found within
    /// the search radius, or [`DomainError::Database`] / [`DomainError::Timeout`]
    /// on repository failure.
    ///
    /// Business logic (CAGR calculation, direction determination) lives here
    /// rather than in the handler layer (P1 review fix).
    #[tracing::instrument(skip(self), fields(usecase = "get_trend"))]
    pub(crate) async fn execute(
        &self,
        coord: Coord,
        years: YearsLookback,
    ) -> Result<TrendAnalysis, DomainError> {
        self.trend_repo
            .find_trend(coord, years)
            .await?
            .ok_or(DomainError::NotFound)
            .and_then(|(location, data)| build_trend_analysis(location, data))
            .inspect(|analysis| {
                tracing::debug!(
                    address = analysis.location.address.as_str(),
                    distance_m = analysis.location.distance_m,
                    data_points = analysis.data.len(),
                    cagr = analysis.cagr,
                    "trend analysis computed"
                )
            })
    }
}

/// Compute CAGR, round, and pick direction to assemble the final analysis.
fn build_trend_analysis(
    location: crate::domain::entity::TrendLocation,
    data: Vec<crate::domain::entity::TrendPoint>,
) -> Result<TrendAnalysis, DomainError> {
    if data.is_empty() {
        tracing::debug!("no trend data found");
        return Err(DomainError::NotFound);
    }

    let first_price = data[0].price_per_sqm.value() as f64;
    let last_price = data[data.len() - 1].price_per_sqm.value() as f64;
    let n_years = (data[data.len() - 1].year.value() - data[0].year.value()).max(1) as u32;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{Address, PricePerSqm, TrendLocation, TrendPoint};
    use crate::domain::repository::mock::MockTrendRepository;
    use crate::domain::value_object::TrendDirection;
    use crate::domain::value_object::Year;

    fn sample_coord() -> Coord {
        Coord::new(35.68, 139.76).unwrap()
    }

    #[test]
    fn trend_direction_as_str() {
        assert_eq!(TrendDirection::Up.as_str(), "up");
        assert_eq!(TrendDirection::Down.as_str(), "down");
    }

    #[tokio::test]
    async fn execute_happy_path_computes_cagr_up() {
        let location = TrendLocation {
            address: Address::parse("Shinjuku").unwrap(),
            distance_m: 42.5,
        };
        let data = vec![
            TrendPoint {
                year: Year::new(2019).unwrap(),
                price_per_sqm: PricePerSqm::new(1000).unwrap(),
            },
            TrendPoint {
                year: Year::new(2023).unwrap(),
                price_per_sqm: PricePerSqm::new(1200).unwrap(),
            },
        ];
        let repo = Arc::new(MockTrendRepository::new().with_find_trend(Ok(Some((location, data)))));
        let usecase = GetTrendUsecase::new(repo);

        let result = usecase
            .execute(sample_coord(), YearsLookback::clamped(5))
            .await
            .unwrap();

        assert_eq!(result.location.address.as_str(), "Shinjuku");
        assert_eq!(result.direction.as_str(), TrendDirection::Up.as_str());
        assert!(result.cagr > 0.0);
    }

    #[tokio::test]
    async fn execute_none_result_returns_not_found() {
        let repo = Arc::new(MockTrendRepository::new().with_find_trend(Ok(None)));
        let usecase = GetTrendUsecase::new(repo);

        let err = usecase
            .execute(sample_coord(), YearsLookback::clamped(5))
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::NotFound));
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let repo = Arc::new(
            MockTrendRepository::new().with_find_trend(Err(DomainError::Database("boom".into()))),
        );
        let usecase = GetTrendUsecase::new(repo);

        let err = usecase
            .execute(sample_coord(), YearsLookback::clamped(5))
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

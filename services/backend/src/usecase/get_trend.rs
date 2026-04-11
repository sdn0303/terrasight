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
    #[tracing::instrument(skip(self), fields(usecase = "get_trend"))]
    pub async fn execute(
        &self,
        coord: Coord,
        years: YearsLookback,
    ) -> Result<TrendAnalysis, DomainError> {
        self.trend_repo
            .find_trend(coord, years)
            .await?
            .ok_or(DomainError::NotFound)
            .and_then(|(location, data)| {
                if data.is_empty() {
                    tracing::debug!("no trend data found");
                    return Err(DomainError::NotFound);
                }
                Ok(build_trend_analysis(location, data))
            })
            .inspect(|analysis| {
                tracing::debug!(
                    address = %analysis.location.address,
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
) -> TrendAnalysis {
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

    TrendAnalysis {
        location,
        data,
        cagr: cagr_rounded,
        direction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::{TrendLocation, TrendPoint};
    use crate::domain::repository::mock::MockTrendRepository;
    use crate::domain::value_object::TrendDirection;

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
            address: "Shinjuku".into(),
            distance_m: 42.5,
        };
        let data = vec![
            TrendPoint {
                year: 2019,
                price_per_sqm: 1000,
            },
            TrendPoint {
                year: 2023,
                price_per_sqm: 1200,
            },
        ];
        let repo = Arc::new(MockTrendRepository::new().with_find_trend(Ok(Some((location, data)))));
        let usecase = GetTrendUsecase::new(repo);

        let result = usecase
            .execute(sample_coord(), YearsLookback::clamped(5))
            .await
            .unwrap();

        assert_eq!(result.location.address, "Shinjuku");
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

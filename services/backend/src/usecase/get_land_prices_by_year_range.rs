use std::sync::Arc;

use crate::domain::entity::LayerResult;
use crate::domain::error::DomainError;
use crate::domain::repository::LandPriceRepository;
use crate::domain::value_object::{BBox, Year, ZoomLevel};

/// Fetch land price GeoJSON features across a `[from_year..=to_year]` range
/// for the time machine animation endpoint.
///
/// The response contains every row with its `properties.year` populated so the
/// frontend can drive a MapLibre `setFilter` slider without additional
/// round-trips.
pub struct GetLandPricesByYearRangeUsecase {
    land_price_repo: Arc<dyn LandPriceRepository>,
}

impl GetLandPricesByYearRangeUsecase {
    pub fn new(land_price_repo: Arc<dyn LandPriceRepository>) -> Self {
        Self { land_price_repo }
    }

    /// Execute the year-range query.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    #[tracing::instrument(skip(self), fields(usecase = "get_land_prices_by_year_range"))]
    pub async fn execute(
        &self,
        from_year: Year,
        to_year: Year,
        bbox: BBox,
        zoom: ZoomLevel,
    ) -> Result<LayerResult, DomainError> {
        self.land_price_repo
            .find_all_years_by_bbox(from_year, to_year, &bbox, zoom)
            .await
            .inspect(|result| {
                tracing::debug!(
                    from_year = from_year.value(),
                    to_year = to_year.value(),
                    feature_count = result.features.len(),
                    truncated = result.truncated,
                    limit = result.limit,
                    "land-prices by-year-range query complete"
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::mock::MockLandPriceRepository;

    fn empty_layer_result() -> LayerResult {
        LayerResult {
            features: Vec::new(),
            truncated: false,
            limit: 500,
        }
    }

    fn sample_bbox() -> BBox {
        BBox::new(35.65, 139.70, 35.70, 139.80).unwrap()
    }

    #[tokio::test]
    async fn execute_happy_path_forwards_repo_result() {
        let repo = Arc::new(
            MockLandPriceRepository::new().with_find_all_years_by_bbox(Ok(empty_layer_result())),
        );
        let usecase = GetLandPricesByYearRangeUsecase::new(repo);

        let result = usecase
            .execute(
                Year::new(2019).unwrap(),
                Year::new(2024).unwrap(),
                sample_bbox(),
                ZoomLevel::clamped(14),
            )
            .await
            .unwrap();

        assert_eq!(result.limit, 500);
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let repo = Arc::new(
            MockLandPriceRepository::new()
                .with_find_all_years_by_bbox(Err(DomainError::Database("boom".into()))),
        );
        let usecase = GetLandPricesByYearRangeUsecase::new(repo);

        let err = usecase
            .execute(
                Year::new(2019).unwrap(),
                Year::new(2024).unwrap(),
                sample_bbox(),
                ZoomLevel::clamped(14),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

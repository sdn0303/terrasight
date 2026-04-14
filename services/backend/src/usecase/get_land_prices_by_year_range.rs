//! Usecase: fetch land price GeoJSON features across a year range.
//!
//! Delegates to [`LandPriceRepository::find_all_years_by_bbox`]. Returns
//! all features with `properties.year` populated so the MapLibre frontend
//! can drive a time-slider without additional round-trips. Called by
//! `GET /api/v1/land-prices/all-years`.

use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::model::{BBox, LayerResult, PrefCode, Year, ZoomLevel};
use crate::domain::repository::LandPriceRepository;

/// Usecase for `GET /api/v1/land-prices/all-years`.
pub(crate) struct GetLandPricesByYearRangeUsecase {
    land_price_repo: Arc<dyn LandPriceRepository>,
}

impl GetLandPricesByYearRangeUsecase {
    /// Construct the usecase with the given repository.
    pub(crate) fn new(land_price_repo: Arc<dyn LandPriceRepository>) -> Self {
        Self { land_price_repo }
    }

    /// Execute the year-range query.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    #[tracing::instrument(skip(self), fields(usecase = "get_land_prices_by_year_range"))]
    pub(crate) async fn execute(
        &self,
        from_year: Year,
        to_year: Year,
        bbox: BBox,
        zoom: ZoomLevel,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        self.land_price_repo
            .find_all_years_by_bbox(from_year, to_year, &bbox, zoom, pref_code)
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
                None,
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
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

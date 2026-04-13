//! Usecase: fetch land price GeoJSON features for a single year.
//!
//! Delegates to [`LandPriceRepository::find_by_year_and_bbox`]. Called by
//! `GET /api/v1/land-prices`.

use std::sync::Arc;

use crate::domain::entity::LayerResult;
use crate::domain::error::DomainError;
use crate::domain::repository::LandPriceRepository;
use crate::domain::value_object::{BBox, PrefCode, Year, ZoomLevel};

/// Usecase for `GET /api/v1/land-prices`.
pub(crate) struct GetLandPricesUsecase {
    land_price_repo: Arc<dyn LandPriceRepository>,
}

impl GetLandPricesUsecase {
    /// Construct the usecase with the given repository.
    pub(crate) fn new(land_price_repo: Arc<dyn LandPriceRepository>) -> Self {
        Self { land_price_repo }
    }

    /// Execute the query and return a [`LayerResult`] with matching features and
    /// truncation metadata.
    ///
    /// `zoom` is forwarded to the repository so that `compute_feature_limit` can
    /// derive the appropriate per-layer cap.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository (typically a database error).
    #[tracing::instrument(skip(self), fields(usecase = "get_land_prices"))]
    pub(crate) async fn execute(
        &self,
        year: Year,
        bbox: BBox,
        zoom: ZoomLevel,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        self.land_price_repo
            .find_by_year_and_bbox(year, &bbox, zoom, pref_code)
            .await
            .inspect(|result| {
                tracing::debug!(
                    year = year.value(),
                    feature_count = result.features.len(),
                    truncated = result.truncated,
                    limit = result.limit,
                    "land-prices query complete"
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
            limit: 100,
        }
    }

    fn sample_bbox() -> BBox {
        BBox::new(35.65, 139.70, 35.70, 139.80).unwrap()
    }

    #[tokio::test]
    async fn execute_happy_path_forwards_repo_result() {
        let repo = Arc::new(
            MockLandPriceRepository::new().with_find_by_year_and_bbox(Ok(empty_layer_result())),
        );
        let usecase = GetLandPricesUsecase::new(repo);

        let result = usecase
            .execute(
                Year::new(2023).unwrap(),
                sample_bbox(),
                ZoomLevel::clamped(14),
                None,
            )
            .await
            .unwrap();

        assert_eq!(result.features.len(), 0);
        assert_eq!(result.limit, 100);
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let repo = Arc::new(
            MockLandPriceRepository::new()
                .with_find_by_year_and_bbox(Err(DomainError::Database("boom".into()))),
        );
        let usecase = GetLandPricesUsecase::new(repo);

        let err = usecase
            .execute(
                Year::new(2023).unwrap(),
                sample_bbox(),
                ZoomLevel::clamped(14),
                None,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

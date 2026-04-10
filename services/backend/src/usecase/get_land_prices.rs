use std::sync::Arc;

use crate::domain::entity::LayerResult;
use crate::domain::error::DomainError;
use crate::domain::repository::LandPriceRepository;
use crate::domain::value_object::{BBox, Year};

/// Fetch land price GeoJSON features for a given year, bounding box, and zoom level.
pub struct GetLandPricesUsecase {
    land_price_repo: Arc<dyn LandPriceRepository>,
}

impl GetLandPricesUsecase {
    pub fn new(land_price_repo: Arc<dyn LandPriceRepository>) -> Self {
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
    pub async fn execute(
        &self,
        year: Year,
        bbox: BBox,
        zoom: u32,
    ) -> Result<LayerResult, DomainError> {
        let result = self
            .land_price_repo
            .find_by_year_and_bbox(&year, &bbox, zoom)
            .await?;

        tracing::debug!(
            year = year.value(),
            feature_count = result.features.len(),
            truncated = result.truncated,
            limit = result.limit,
            "land-prices query complete"
        );

        Ok(result)
    }

    /// Execute the all-years query for time machine animation.
    ///
    /// Returns features with `year` property so the client can filter client-side
    /// without refetching on every year tick.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    pub async fn execute_all_years(
        &self,
        from_year: Year,
        to_year: Year,
        bbox: BBox,
        zoom: u32,
    ) -> Result<LayerResult, DomainError> {
        let result = self
            .land_price_repo
            .find_all_years_by_bbox(&from_year, &to_year, &bbox, zoom)
            .await?;

        tracing::debug!(
            from_year = from_year.value(),
            to_year = to_year.value(),
            feature_count = result.features.len(),
            truncated = result.truncated,
            limit = result.limit,
            "land-prices all-years query complete"
        );

        Ok(result)
    }
}

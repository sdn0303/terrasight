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

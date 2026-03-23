use std::sync::Arc;

use crate::domain::entity::GeoFeature;
use crate::domain::error::DomainError;
use crate::domain::repository::LandPriceRepository;
use crate::domain::value_object::{BBox, Year};

/// Fetch land price GeoJSON features for a given year and bounding box.
pub struct GetLandPricesUsecase {
    land_price_repo: Arc<dyn LandPriceRepository>,
}

impl GetLandPricesUsecase {
    pub fn new(land_price_repo: Arc<dyn LandPriceRepository>) -> Self {
        Self { land_price_repo }
    }

    /// Execute the query and return matching [`GeoFeature`] items.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository (typically a database error).
    pub async fn execute(&self, year: Year, bbox: BBox) -> Result<Vec<GeoFeature>, DomainError> {
        let features = self
            .land_price_repo
            .find_by_year_and_bbox(&year, &bbox)
            .await?;

        tracing::debug!(
            year = year.value(),
            feature_count = features.len(),
            "land-prices query complete"
        );

        Ok(features)
    }
}

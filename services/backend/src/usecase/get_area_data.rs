use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::GeoFeature;
use crate::domain::error::DomainError;
use crate::domain::repository::AreaRepository;
use crate::domain::value_object::{BBox, LayerType};

pub struct GetAreaDataUsecase {
    area_repo: Arc<dyn AreaRepository>,
}

impl GetAreaDataUsecase {
    pub fn new(area_repo: Arc<dyn AreaRepository>) -> Self {
        Self { area_repo }
    }

    /// Fetch GeoJSON features for the requested layers within the bounding box.
    ///
    /// Layers are queried in parallel via `tokio::try_join!` when multiple are
    /// requested (P1 review fix: avoid sequential execution).
    pub async fn execute(
        &self,
        bbox: &BBox,
        layers: &[LayerType],
    ) -> Result<HashMap<LayerType, Vec<GeoFeature>>, DomainError> {
        if layers.is_empty() {
            return Err(DomainError::MissingParameter("layers".into()));
        }

        // Build futures for each requested layer. We collect into a Vec of
        // (LayerType, Future) pairs, then join them all concurrently.
        let futures: Vec<_> = layers
            .iter()
            .map(|layer| {
                let repo = Arc::clone(&self.area_repo);
                let bbox = bbox.clone();
                let layer = *layer;
                async move {
                    let features = match layer {
                        LayerType::LandPrice => repo.find_land_prices(&bbox).await,
                        LayerType::Zoning => repo.find_zoning(&bbox).await,
                        LayerType::Flood => repo.find_flood_risk(&bbox).await,
                        LayerType::SteepSlope => repo.find_steep_slope(&bbox).await,
                        LayerType::Schools => repo.find_schools(&bbox).await,
                        LayerType::Medical => repo.find_medical(&bbox).await,
                    }?;
                    tracing::debug!(
                        layer = layer.as_str(),
                        row_count = features.len(),
                        "layer rows fetched"
                    );
                    Ok::<_, DomainError>((layer, features))
                }
            })
            .collect();

        let results = futures::future::try_join_all(futures).await?;
        Ok(results.into_iter().collect())
    }
}

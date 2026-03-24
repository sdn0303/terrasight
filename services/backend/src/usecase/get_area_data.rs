use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::LayerResult;
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
    /// `zoom` is forwarded to each repository method so that dynamic per-layer
    /// limits can be computed via `compute_feature_limit`.
    ///
    /// Layers are queried in parallel via `futures::future::try_join_all` (P1
    /// review fix: avoid sequential execution).
    pub async fn execute(
        &self,
        bbox: &BBox,
        layers: &[LayerType],
        zoom: u32,
    ) -> Result<HashMap<LayerType, LayerResult>, DomainError> {
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
                    let result = match layer {
                        LayerType::LandPrice => repo.find_land_prices(&bbox, zoom).await,
                        LayerType::Zoning => repo.find_zoning(&bbox, zoom).await,
                        LayerType::Flood => repo.find_flood_risk(&bbox, zoom).await,
                        LayerType::SteepSlope => repo.find_steep_slope(&bbox, zoom).await,
                        LayerType::Schools => repo.find_schools(&bbox, zoom).await,
                        LayerType::Medical => repo.find_medical(&bbox, zoom).await,
                    }?;
                    tracing::debug!(
                        layer = layer.as_str(),
                        row_count = result.features.len(),
                        truncated = result.truncated,
                        limit = result.limit,
                        "layer rows fetched"
                    );
                    Ok::<_, DomainError>((layer, result))
                }
            })
            .collect();

        let results = futures::future::try_join_all(futures).await?;
        Ok(results.into_iter().collect())
    }
}

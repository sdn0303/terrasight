use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::LayerResult;
use crate::domain::error::DomainError;
use crate::domain::repository::LayerRepository;
use crate::domain::value_object::{BBox, LayerType};

pub struct GetAreaDataUsecase {
    layer_repo: Arc<dyn LayerRepository>,
}

impl GetAreaDataUsecase {
    pub fn new(layer_repo: Arc<dyn LayerRepository>) -> Self {
        Self { layer_repo }
    }

    /// Fetch GeoJSON features for the requested layers within the bounding box.
    ///
    /// Layers are queried in parallel via `futures::future::try_join_all` so
    /// that the total latency is `max(layer_latency)` rather than `sum`.
    pub async fn execute(
        &self,
        bbox: &BBox,
        layers: &[LayerType],
        zoom: u32,
    ) -> Result<HashMap<LayerType, LayerResult>, DomainError> {
        if layers.is_empty() {
            return Err(DomainError::MissingParameter("layers".into()));
        }

        let futures = layers.iter().map(|layer| {
            let repo = Arc::clone(&self.layer_repo);
            let bbox = *bbox;
            let layer = *layer;
            async move {
                let result = repo.find_layer(layer, &bbox, zoom).await.inspect(|r| {
                    tracing::debug!(
                        layer = layer.as_str(),
                        row_count = r.features.len(),
                        truncated = r.truncated,
                        limit = r.limit,
                        "layer rows fetched"
                    )
                })?;
                Ok::<_, DomainError>((layer, result))
            }
        });

        let results = futures::future::try_join_all(futures).await?;
        Ok(results.into_iter().collect())
    }
}

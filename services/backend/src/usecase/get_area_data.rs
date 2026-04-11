use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::LayerResult;
use crate::domain::error::DomainError;
use crate::domain::repository::LayerRepository;
use crate::domain::value_object::{BBox, LayerType, ZoomLevel};

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
    #[tracing::instrument(skip(self), fields(usecase = "get_area_data", layer_count = layers.len()))]
    pub async fn execute(
        &self,
        bbox: &BBox,
        layers: &[LayerType],
        zoom: ZoomLevel,
    ) -> Result<HashMap<LayerType, LayerResult>, DomainError> {
        if layers.is_empty() {
            return Err(DomainError::MissingParameter("layers".into()));
        }

        let futures = layers.iter().map(|layer| {
            let repo = Arc::clone(&self.layer_repo);
            let bbox = *bbox;
            let layer = *layer;
            async move {
                repo.find_layer(layer, &bbox, zoom)
                    .await
                    .inspect(|r| {
                        tracing::debug!(
                            layer = layer.as_str(),
                            row_count = r.features.len(),
                            truncated = r.truncated,
                            limit = r.limit,
                            "layer rows fetched"
                        )
                    })
                    .map(|result| (layer, result))
            }
        });

        futures::future::try_join_all(futures)
            .await
            .map(|results| results.into_iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::repository::mock::MockLayerRepository;

    fn sample_bbox() -> BBox {
        BBox::new(35.65, 139.70, 35.70, 139.80).unwrap()
    }

    fn layer_result_with(limit: i64) -> LayerResult {
        LayerResult {
            features: Vec::new(),
            truncated: false,
            limit,
        }
    }

    #[tokio::test]
    async fn execute_multi_layer_aggregation_returns_all_layers() {
        // Queue three mocked responses in the same order as the request layers
        // below so `try_join_all` can consume them concurrently.
        let repo = Arc::new(
            MockLayerRepository::new()
                .with_find_layer(Ok(layer_result_with(100)))
                .with_find_layer(Ok(layer_result_with(200)))
                .with_find_layer(Ok(layer_result_with(300))),
        );
        let usecase = GetAreaDataUsecase::new(repo);
        let layers = [LayerType::LandPrice, LayerType::Zoning, LayerType::Flood];

        let result = usecase
            .execute(&sample_bbox(), &layers, ZoomLevel::clamped(14))
            .await
            .unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.contains_key(&LayerType::LandPrice));
        assert!(result.contains_key(&LayerType::Zoning));
        assert!(result.contains_key(&LayerType::Flood));
    }

    #[tokio::test]
    async fn execute_rejects_empty_layers() {
        let repo = Arc::new(MockLayerRepository::new());
        let usecase = GetAreaDataUsecase::new(repo);

        let err = usecase
            .execute(&sample_bbox(), &[], ZoomLevel::clamped(14))
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::MissingParameter(_)));
    }

    #[tokio::test]
    async fn execute_propagates_db_error() {
        let repo = Arc::new(
            MockLayerRepository::new().with_find_layer(Err(DomainError::Database("boom".into()))),
        );
        let usecase = GetAreaDataUsecase::new(repo);

        let err = usecase
            .execute(
                &sample_bbox(),
                &[LayerType::LandPrice],
                ZoomLevel::clamped(14),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, DomainError::Database(_)));
    }
}

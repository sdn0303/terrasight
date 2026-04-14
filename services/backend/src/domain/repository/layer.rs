//! [`LayerRepository`] trait — GeoJSON feature queries by map layer and bounding box.

use async_trait::async_trait;

use crate::domain::error::DomainError;
use crate::domain::model::{BBox, LayerResult, LayerType, PrefCode, ZoomLevel};

/// Repository for map layer GeoJSON features.
///
/// Uses a single enum-dispatched entry point ([`find_layer`]) so the usecase
/// can fan out over all [`LayerType`] variants concurrently without the trait
/// growing a new method each time a layer is added.
///
/// Implemented by `PgLayerRepository` in the `infra` layer.
///
/// [`find_layer`]: LayerRepository::find_layer
#[async_trait]
pub trait LayerRepository: Send + Sync {
    /// Fetch GeoJSON features for a single map layer within the given bbox.
    ///
    /// The feature count is capped by a zoom-dependent limit; [`LayerResult`]
    /// carries a `truncated` flag when the cap was hit.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on SQL failure.
    async fn find_layer(
        &self,
        layer: LayerType,
        bbox: &BBox,
        zoom: ZoomLevel,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError>;
}

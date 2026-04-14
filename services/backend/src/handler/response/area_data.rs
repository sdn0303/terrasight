//! Response DTO for `GET /api/v1/area-data`.
//!
//! Wraps a `HashMap<String, LayerResponseDto>` so each requested layer
//! is serialised as a top-level JSON key whose value is a
//! `LayerResponseDto` FeatureCollection.

use std::collections::HashMap;

use serde::Serialize;

use crate::domain::entity::LayerResult;
use crate::domain::value_object::LayerType;
use crate::handler::response::LayerResponseDto;

/// Flat object keyed by layer name, one `LayerResponseDto` per layer.
///
/// Using `#[serde(transparent)]` lets this wrap a `HashMap<String, _>` on the
/// wire without the top-level `{"layers": …}` envelope, preserving backwards
/// compatibility with the pre-refactor `Json<serde_json::Value>` response.
#[derive(Debug, Serialize)]
#[serde(transparent)]
pub struct AreaDataResponseDto {
    pub layers: HashMap<String, LayerResponseDto>,
}

impl AreaDataResponseDto {
    /// Build an [`AreaDataResponseDto`] from the usecase output, running
    /// `LandPrice` features through `point_feature_to_polygon` via
    /// [`LayerResponseDto::from_layer_result`].
    pub fn from_domain(result: HashMap<LayerType, LayerResult>) -> Self {
        let layers = result
            .into_iter()
            .map(|(layer, layer_result)| {
                (
                    layer.as_str().to_string(),
                    LayerResponseDto::from_layer_result(layer_result, layer),
                )
            })
            .collect();
        Self { layers }
    }
}

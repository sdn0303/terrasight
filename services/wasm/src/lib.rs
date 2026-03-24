//! # realestate-wasm
//!
//! WebAssembly spatial engine for the real estate investment data visualiser.
//!
//! Exposes a [`SpatialEngine`] to JavaScript via `wasm-bindgen`. The engine
//! ingests FlatGeobuf (`*.fgb`) byte arrays, builds an in-memory R-tree per
//! layer, and returns GeoJSON `FeatureCollection` strings for MapLibre GL
//! rendering.
//!
//! ## Usage (JavaScript/TypeScript)
//!
//! ```js
//! import init, { SpatialEngine } from '/wasm/realestate_wasm.js';
//! await init();
//!
//! const engine = new SpatialEngine();
//! const fgbBytes = new Uint8Array(await fetch('/data/geology.fgb').then(r => r.arrayBuffer()));
//! const count = engine.load_layer('geology', fgbBytes);
//! console.log(`Loaded ${count} features`);
//!
//! // Query Tokyo 23ku
//! const geojson = engine.query('geology', 35.53, 139.57, 35.82, 139.92);
//! ```

use std::collections::HashMap;

use wasm_bindgen::prelude::*;

mod fgb_reader;
mod spatial_index;

use fgb_reader::parse_fgb;
use spatial_index::LayerIndex;

/// Multi-layer spatial engine exposed to JavaScript.
///
/// Each layer is independently loaded from a FlatGeobuf byte array and stored
/// as an R-tree. Queries are strictly read-only after loading.
#[wasm_bindgen]
pub struct SpatialEngine {
    layers: HashMap<String, LayerIndex>,
}

#[wasm_bindgen]
impl SpatialEngine {
    /// Create an empty [`SpatialEngine`].
    ///
    /// # Examples
    ///
    /// ```js
    /// const engine = new SpatialEngine();
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            layers: HashMap::new(),
        }
    }

    /// Parse `fgb_bytes` and load them into an R-tree under `layer_id`.
    ///
    /// Replaces any existing layer with the same `layer_id`.
    ///
    /// Returns the number of features loaded.
    ///
    /// # Errors
    ///
    /// Propagates parse errors from [`parse_fgb`] as JavaScript `Error` objects.
    pub fn load_layer(&mut self, layer_id: &str, fgb_bytes: &[u8]) -> Result<u32, JsValue> {
        self.load_layer_inner(layer_id, fgb_bytes)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Query a single layer by bounding box.
    ///
    /// Returns a GeoJSON `FeatureCollection` string with all features whose
    /// envelopes intersect `[west, south, east, north]`.
    ///
    /// Coordinates follow RFC 7946 `[longitude, latitude]`.
    ///
    /// # Errors
    ///
    /// Returns a JavaScript `Error` if `layer_id` has not been loaded.
    pub fn query(
        &self,
        layer_id: &str,
        south: f64,
        west: f64,
        north: f64,
        east: f64,
    ) -> Result<String, JsValue> {
        self.query_inner(layer_id, south, west, north, east)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Query multiple layers simultaneously.
    ///
    /// `layer_ids` is a comma-separated string of layer identifiers, e.g.
    /// `"geology,landform"`.
    ///
    /// Returns a JSON object keyed by layer id, where each value is a GeoJSON
    /// `FeatureCollection` string.
    ///
    /// ```json
    /// {
    ///   "geology":  "{\"type\":\"FeatureCollection\",\"features\":[...]}",
    ///   "landform": "{\"type\":\"FeatureCollection\",\"features\":[]}"
    /// }
    /// ```
    ///
    /// Layers that have not been loaded are silently omitted from the result.
    ///
    /// # Errors
    ///
    /// Returns a JavaScript `Error` if JSON serialisation of the result fails
    /// (should never happen in practice).
    pub fn query_layers(
        &self,
        layer_ids: &str,
        south: f64,
        west: f64,
        north: f64,
        east: f64,
    ) -> Result<String, JsValue> {
        self.query_layers_inner(layer_ids, south, west, north, east)
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Return the number of features in the specified layer, or `0` if the
    /// layer has not been loaded.
    pub fn feature_count(&self, layer_id: &str) -> u32 {
        self.layers
            .get(layer_id)
            .map(LayerIndex::feature_count)
            .unwrap_or(0)
    }

    /// Return a JSON array string of all loaded layer ids.
    ///
    /// Example: `["geology","landform"]`
    pub fn loaded_layers(&self) -> String {
        let ids: Vec<&str> = self.layers.keys().map(String::as_str).collect();
        // serde_json is infallible for Vec<&str>
        serde_json::to_string(&ids).unwrap_or_else(|_| "[]".to_string())
    }
}

impl SpatialEngine {
    /// Internal load implementation returning `Result<_, String>` for testability
    /// without `JsValue` (which panics on non-wasm32 targets).
    pub fn load_layer_inner(
        &mut self,
        layer_id: &str,
        fgb_bytes: &[u8],
    ) -> Result<u32, String> {
        let features = parse_fgb(fgb_bytes)?;
        let count = features.len() as u32;
        let index = LayerIndex::from_parsed(features);
        self.layers.insert(layer_id.to_string(), index);
        Ok(count)
    }

    /// Internal query implementation returning `Result<_, String>`.
    pub fn query_inner(
        &self,
        layer_id: &str,
        south: f64,
        west: f64,
        north: f64,
        east: f64,
    ) -> Result<String, String> {
        let index = self
            .layers
            .get(layer_id)
            .ok_or_else(|| format!("layer not found: {layer_id}"))?;

        let indices = index.query_bbox(south, west, north, east);
        Ok(index.get_features_geojson(&indices))
    }

    /// Internal query_layers implementation returning `Result<_, String>`.
    pub fn query_layers_inner(
        &self,
        layer_ids: &str,
        south: f64,
        west: f64,
        north: f64,
        east: f64,
    ) -> Result<String, String> {
        let mut result: HashMap<&str, String> = HashMap::new();

        for layer_id in layer_ids.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            if let Some(index) = self.layers.get(layer_id) {
                let indices = index.query_bbox(south, west, north, east);
                result.insert(layer_id, index.get_features_geojson(&indices));
            }
        }

        serde_json::to_string(&result)
            .map_err(|e| format!("JSON serialisation error: {e}"))
    }
}

impl Default for SpatialEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FGB_PATH: &str = "../../data/fgb/13/geology.fgb";
    const FGB_PATH_2: &str = "../../data/fgb/13/landform.fgb";

    fn geology_bytes() -> Vec<u8> {
        std::fs::read(FGB_PATH).expect("geology.fgb should exist at data/fgb/13/")
    }

    fn landform_bytes() -> Vec<u8> {
        std::fs::read(FGB_PATH_2).expect("landform.fgb should exist at data/fgb/13/")
    }

    // -------------------------------------------------------------------------
    // SpatialEngine::new
    // -------------------------------------------------------------------------

    #[test]
    fn new_engine_has_no_layers() {
        let engine = SpatialEngine::new();
        let layers_json = engine.loaded_layers();
        let layers: serde_json::Value = serde_json::from_str(&layers_json).unwrap();
        assert_eq!(layers.as_array().unwrap().len(), 0);
    }

    // -------------------------------------------------------------------------
    // SpatialEngine::load_layer_inner
    // -------------------------------------------------------------------------

    #[test]
    fn load_layer_returns_feature_count() {
        let mut engine = SpatialEngine::new();
        let count = engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("load_layer_inner should succeed");
        assert_eq!(count, 133, "geology.fgb should have 133 features");
    }

    #[test]
    fn load_layer_registers_layer_id() {
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("load_layer_inner should succeed");

        let layers_json = engine.loaded_layers();
        let layers: Vec<String> = serde_json::from_str(&layers_json).unwrap();
        assert!(layers.contains(&"geology".to_string()));
    }

    #[test]
    fn load_layer_invalid_bytes_returns_err() {
        let mut engine = SpatialEngine::new();
        let result = engine.load_layer_inner("bad", b"not a fgb file");
        assert!(result.is_err(), "invalid bytes should return Err");
    }

    #[test]
    fn load_layer_replaces_existing_layer() {
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("first load should succeed");
        let count2 = engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("second load should succeed");
        assert_eq!(count2, 133);
        assert_eq!(engine.feature_count("geology"), 133);
    }

    // -------------------------------------------------------------------------
    // SpatialEngine::query_inner
    // -------------------------------------------------------------------------

    #[test]
    fn query_tokyo_bbox_returns_nonempty_feature_collection() {
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("load should succeed");

        let geojson = engine
            .query_inner("geology", 35.53, 139.57, 35.82, 139.92)
            .expect("query_inner should succeed");

        let parsed: serde_json::Value = serde_json::from_str(&geojson).unwrap();
        assert_eq!(parsed["type"], "FeatureCollection");
        let features = parsed["features"].as_array().unwrap();
        assert!(!features.is_empty(), "Tokyo bbox should return features");
    }

    #[test]
    fn query_outside_data_returns_empty_feature_collection() {
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("load should succeed");

        let geojson = engine
            .query_inner("geology", 51.3, -0.5, 51.7, 0.3) // London
            .expect("query_inner should succeed");

        let parsed: serde_json::Value = serde_json::from_str(&geojson).unwrap();
        let features = parsed["features"].as_array().unwrap();
        assert!(features.is_empty(), "London bbox should return no features");
    }

    #[test]
    fn query_unknown_layer_returns_err() {
        let engine = SpatialEngine::new();
        let result = engine.query_inner("nonexistent", 35.5, 139.5, 35.9, 140.0);
        assert!(result.is_err(), "unknown layer should return Err");
    }

    // -------------------------------------------------------------------------
    // SpatialEngine::query_layers_inner
    // -------------------------------------------------------------------------

    #[test]
    fn query_layers_multiple_layers() {
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("geology load should succeed");
        engine
            .load_layer_inner("landform", &landform_bytes())
            .expect("landform load should succeed");

        let result_json = engine
            .query_layers_inner("geology,landform", 35.53, 139.57, 35.82, 139.92)
            .expect("query_layers_inner should succeed");

        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
        assert!(result.get("geology").is_some(), "result should contain geology");
        assert!(result.get("landform").is_some(), "result should contain landform");
    }

    #[test]
    fn query_layers_omits_unloaded_layers() {
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("geology load should succeed");

        let result_json = engine
            .query_layers_inner("geology,not_loaded", 35.53, 139.57, 35.82, 139.92)
            .expect("query_layers_inner should succeed");

        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
        assert!(result.get("geology").is_some());
        assert!(
            result.get("not_loaded").is_none(),
            "unloaded layer should be omitted"
        );
    }

    #[test]
    fn query_layers_each_value_is_feature_collection() {
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("load should succeed");

        let result_json = engine
            .query_layers_inner("geology", 35.53, 139.57, 35.82, 139.92)
            .expect("query_layers_inner should succeed");

        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
        let fc_str = result["geology"].as_str().unwrap();
        let fc: serde_json::Value = serde_json::from_str(fc_str).unwrap();
        assert_eq!(fc["type"], "FeatureCollection");
    }

    // -------------------------------------------------------------------------
    // SpatialEngine::feature_count
    // -------------------------------------------------------------------------

    #[test]
    fn feature_count_unloaded_layer_returns_zero() {
        let engine = SpatialEngine::new();
        assert_eq!(engine.feature_count("geology"), 0);
    }

    #[test]
    fn feature_count_after_load_matches_parse_count() {
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("load should succeed");
        assert_eq!(engine.feature_count("geology"), 133);
    }
}

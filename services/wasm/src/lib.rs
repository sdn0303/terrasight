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
//!
//! // Compute area statistics
//! const stats = engine.compute_stats(35.53, 139.57, 35.82, 139.92);
//! ```

use std::collections::HashMap;

use geo::{Coord, Rect};
use wasm_bindgen::prelude::*;

mod fgb_reader;
mod spatial_index;
mod stats;

use fgb_reader::parse_fgb;
use spatial_index::{LayerIndex, LayerStatsData};
use stats::{
    compute_area_ratio, compute_land_price_stats, compute_zoning_distribution,
};

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
    /// `FeatureCollection` object.
    ///
    /// ```json
    /// {
    ///   "geology":  {"type":"FeatureCollection","features":[...]},
    ///   "landform": {"type":"FeatureCollection","features":[]}
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

    /// Compute area statistics for the given bounding box.
    ///
    /// Returns a JSON string matching the backend `/api/stats` response shape:
    ///
    /// ```json
    /// {
    ///   "land_price": {
    ///     "avg_per_sqm": 850000,
    ///     "median_per_sqm": 720000,
    ///     "min_per_sqm": 320000,
    ///     "max_per_sqm": 3200000,
    ///     "count": 45
    ///   },
    ///   "risk": {
    ///     "flood_area_ratio": 0.15,
    ///     "steep_slope_area_ratio": 0.02,
    ///     "composite_risk": 0.10
    ///   },
    ///   "facilities": { "schools": 12, "medical": 28 },
    ///   "zoning_distribution": { "商業地域": 0.35, "住居地域": 0.45 }
    /// }
    /// ```
    ///
    /// Missing layers degrade gracefully to zero / empty values.
    ///
    /// Coordinates follow RFC 7946 `[longitude, latitude]`.
    ///
    /// # Errors
    ///
    /// Returns a JavaScript `Error` if JSON serialisation fails (should not
    /// occur in practice).
    pub fn compute_stats(
        &self,
        south: f64,
        west: f64,
        north: f64,
        east: f64,
    ) -> Result<String, JsValue> {
        self.compute_stats_inner(south, west, north, east)
            .map_err(|e| JsValue::from_str(&e))
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
        let index = LayerIndex::from_parsed(features, layer_id);
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
        let mut result: HashMap<&str, serde_json::Value> = HashMap::new();

        for layer_id in layer_ids.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            if let Some(index) = self.layers.get(layer_id) {
                let indices = index.query_bbox(south, west, north, east);
                result.insert(layer_id, index.get_features_as_value(&indices));
            }
        }

        serde_json::to_string(&result)
            .map_err(|e| format!("JSON serialisation error: {e}"))
    }

    /// Internal compute_stats implementation returning `Result<_, String>`.
    pub fn compute_stats_inner(
        &self,
        south: f64,
        west: f64,
        north: f64,
        east: f64,
    ) -> Result<String, String> {
        let bbox_rect = Rect::new(
            Coord { x: west, y: south },
            Coord { x: east, y: north },
        );

        // --- Land price ---
        let lp_stats = self
            .layers
            .get("landprice")
            .map(|idx| {
                let indices = idx.query_bbox(south, west, north, east);
                compute_land_price_stats(&idx.stats_data, &indices)
            })
            .unwrap_or_else(|| {
                compute_land_price_stats(&LayerStatsData::None, &[])
            });

        // --- Flood risk ---
        let flood_ratio = self
            .find_layer_ratio(&bbox_rect, south, west, north, east, &["flood-history", "flood"]);

        // --- Steep slope risk ---
        let steep_ratio = self
            .find_layer_ratio(&bbox_rect, south, west, north, east, &["steep-slope", "steep_slope"]);

        // --- Composite risk ---
        let composite = (flood_ratio * RISK_WEIGHT_FLOOD + steep_ratio * RISK_WEIGHT_STEEP)
            .clamp(0.0, 1.0);

        // --- Schools ---
        let schools = self
            .layers
            .get("schools")
            .map(|idx| idx.query_bbox(south, west, north, east).len() as u32)
            .unwrap_or(0);

        // --- Medical ---
        let medical = self
            .layers
            .get("medical")
            .map(|idx| idx.query_bbox(south, west, north, east).len() as u32)
            .unwrap_or(0);

        // --- Zoning distribution ---
        let zoning_dist = self
            .layers
            .get("zoning")
            .map(|idx| {
                let indices = idx.query_bbox(south, west, north, east);
                compute_zoning_distribution(&bbox_rect, &idx.stats_data, &indices)
            })
            .unwrap_or_default();

        // Build zoning_distribution as a JSON object (zone_type -> ratio).
        let zoning_obj: serde_json::Map<String, serde_json::Value> = zoning_dist
            .into_iter()
            .map(|(zone, ratio)| (zone, serde_json::Value::from(ratio)))
            .collect();

        let response = serde_json::json!({
            "land_price": {
                "avg_per_sqm": lp_stats.avg_per_sqm,
                "median_per_sqm": lp_stats.median_per_sqm,
                "min_per_sqm": lp_stats.min_per_sqm,
                "max_per_sqm": lp_stats.max_per_sqm,
                "count": lp_stats.count,
            },
            "risk": {
                "flood_area_ratio": flood_ratio,
                "steep_slope_area_ratio": steep_ratio,
                "composite_risk": composite,
            },
            "facilities": {
                "schools": schools,
                "medical": medical,
            },
            "zoning_distribution": serde_json::Value::Object(zoning_obj),
        });

        serde_json::to_string(&response)
            .map_err(|e| format!("JSON serialisation error: {e}"))
    }

    /// Query the first matching layer from `candidates` and compute its area ratio.
    ///
    /// Returns `0.0` if none of the candidate layer ids are loaded.
    fn find_layer_ratio(
        &self,
        bbox_rect: &Rect<f64>,
        south: f64,
        west: f64,
        north: f64,
        east: f64,
        candidates: &[&str],
    ) -> f64 {
        for &layer_id in candidates {
            if let Some(idx) = self.layers.get(layer_id) {
                let indices = idx.query_bbox(south, west, north, east);
                return compute_area_ratio(bbox_rect, &idx.stats_data, &indices);
            }
        }
        0.0
    }
}

/// Risk weight for flood area ratio (mirrors backend `domain/constants.rs`).
const RISK_WEIGHT_FLOOD: f64 = 0.6;

/// Risk weight for steep-slope area ratio (mirrors backend `domain/constants.rs`).
const RISK_WEIGHT_STEEP: f64 = 0.4;

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
        let fc = &result["geology"];
        assert_eq!(fc["type"], "FeatureCollection");
        assert!(fc["features"].is_array());
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

    // -------------------------------------------------------------------------
    // SpatialEngine::compute_stats_inner
    // -------------------------------------------------------------------------

    #[test]
    fn test_compute_stats_empty_engine_returns_all_zeros() {
        let engine = SpatialEngine::new();
        let json = engine
            .compute_stats_inner(35.53, 139.57, 35.82, 139.92)
            .expect("compute_stats_inner should succeed on empty engine");

        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(v["land_price"]["count"], 0);
        assert_eq!(v["land_price"]["avg_per_sqm"], 0.0);
        assert_eq!(v["risk"]["flood_area_ratio"], 0.0);
        assert_eq!(v["risk"]["steep_slope_area_ratio"], 0.0);
        assert_eq!(v["risk"]["composite_risk"], 0.0);
        assert_eq!(v["facilities"]["schools"], 0);
        assert_eq!(v["facilities"]["medical"], 0);
        assert!(
            v["zoning_distribution"].as_object().unwrap().is_empty(),
            "zoning_distribution should be empty"
        );
    }

    #[test]
    fn test_compute_stats_with_loaded_geology_layer_no_panic() {
        // Geology is not a stats-relevant layer; verify graceful degradation.
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("load should succeed");

        let json = engine
            .compute_stats_inner(35.53, 139.57, 35.82, 139.92)
            .expect("compute_stats_inner should not panic with geology loaded");

        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        // All stats keys must be present even without stats-relevant layers.
        assert!(v.get("land_price").is_some());
        assert!(v.get("risk").is_some());
        assert!(v.get("facilities").is_some());
        assert!(v.get("zoning_distribution").is_some());
    }

    #[test]
    fn test_compute_stats_response_has_all_required_keys() {
        let engine = SpatialEngine::new();
        let json = engine
            .compute_stats_inner(35.53, 139.57, 35.82, 139.92)
            .expect("compute_stats_inner should succeed");

        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify top-level keys
        for key in &["land_price", "risk", "facilities", "zoning_distribution"] {
            assert!(v.get(key).is_some(), "missing top-level key: {key}");
        }

        // Verify land_price sub-keys
        for key in &["avg_per_sqm", "median_per_sqm", "min_per_sqm", "max_per_sqm", "count"] {
            assert!(
                v["land_price"].get(key).is_some(),
                "missing land_price key: {key}"
            );
        }

        // Verify risk sub-keys
        for key in &["flood_area_ratio", "steep_slope_area_ratio", "composite_risk"] {
            assert!(v["risk"].get(key).is_some(), "missing risk key: {key}");
        }

        // Verify facilities sub-keys
        for key in &["schools", "medical"] {
            assert!(v["facilities"].get(key).is_some(), "missing facilities key: {key}");
        }
    }

    #[test]
    fn test_compute_stats_flood_history_layer_contributes_to_risk() {
        // Load geology as flood-history to exercise area ratio path.
        // Geology contains polygon data so this exercises the code path.
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("flood-history", &geology_bytes())
            .expect("load should succeed");

        let json = engine
            .compute_stats_inner(35.53, 139.57, 35.82, 139.92)
            .expect("compute_stats_inner should succeed");

        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let flood_ratio = v["risk"]["flood_area_ratio"].as_f64().unwrap();
        let composite = v["risk"]["composite_risk"].as_f64().unwrap();

        // Ratios must be in valid range
        assert!((0.0..=1.0).contains(&flood_ratio), "flood_ratio out of range: {flood_ratio}");
        assert!((0.0..=1.0).contains(&composite), "composite out of range: {composite}");
    }

    #[test]
    fn test_compute_stats_schools_layer_counts_hits() {
        let mut engine = SpatialEngine::new();
        // Load geology as "schools" — geology has features in Tokyo bbox,
        // so we expect a non-zero count.
        engine
            .load_layer_inner("schools", &geology_bytes())
            .expect("load should succeed");

        let json = engine
            .compute_stats_inner(35.53, 139.57, 35.82, 139.92)
            .expect("compute_stats_inner should succeed");

        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let school_count = v["facilities"]["schools"].as_u64().unwrap();
        assert!(school_count > 0, "should count schools in Tokyo bbox");
    }
}

//! # terrasight-wasm
//!
//! WebAssembly spatial engine for the Terrasight data visualisation platform.
//!
//! Exposes a [`SpatialEngine`] to JavaScript via `wasm-bindgen`. The engine
//! ingests FlatGeobuf (`*.fgb`) byte arrays, builds an in-memory R-tree per
//! layer, and returns GeoJSON `FeatureCollection` strings for MapLibre GL
//! rendering.
//!
//! ## Usage (JavaScript/TypeScript)
//!
//! ```js
//! import init, { SpatialEngine } from '/wasm/terrasight_wasm.js';
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

mod bbox;
mod constants;
mod error;
mod fgb_reader;
mod spatial_index;
mod stats;
mod tls;

pub use bbox::BBox;
pub use error::WasmError;

use fgb_reader::parse_fgb;
use spatial_index::{LayerIndex, LayerStatsData};
use stats::{
    AreaStats, FacilityStats, RiskStats, ZoningEntry, compute_area_ratio, compute_land_price_stats,
    compute_zoning_distribution,
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
    /// Propagates parse errors from `parse_fgb` as JavaScript `Error` objects.
    pub fn load_layer(&mut self, layer_id: &str, fgb_bytes: &[u8]) -> Result<u32, JsValue> {
        let layer_id = constants::canonical_layer_id(layer_id);
        self.load_layer_inner(&layer_id, fgb_bytes)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Parse `geojson` as a GeoJSON FeatureCollection and load it into an R-tree under `layer_id`.
    ///
    /// Used for API-fetched layers that are not served as FlatGeobuf.
    /// Replaces any existing layer with the same `layer_id`.
    ///
    /// Returns the number of features loaded.
    ///
    /// # Errors
    ///
    /// Propagates parse errors as JavaScript `Error` objects.
    pub fn load_geojson_layer(&mut self, layer_id: &str, geojson: &str) -> Result<u32, JsValue> {
        let layer_id = constants::canonical_layer_id(layer_id);
        self.load_geojson_layer_inner(&layer_id, geojson)
            .map_err(|e| JsValue::from_str(&e.to_string()))
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
        let bbox =
            BBox::new(south, west, north, east).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let layer_id = constants::canonical_layer_id(layer_id);
        self.query_inner(&bbox, &layer_id)
            .map_err(|e| JsValue::from_str(&e.to_string()))
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
        let bbox =
            BBox::new(south, west, north, east).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.query_layers_inner(&bbox, layer_ids)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Return the number of features in the specified layer, or `0` if the
    /// layer has not been loaded.
    ///
    /// The `layer_id` is normalised before lookup — both `"land-price"` and
    /// `"landprice"` resolve to the same layer.
    pub fn feature_count(&self, layer_id: &str) -> u32 {
        let layer_id = constants::canonical_layer_id(layer_id);
        self.layers
            .get(&layer_id)
            .map(LayerIndex::feature_count)
            .unwrap_or(0)
    }

    /// Return a JSON array string of all loaded layer ids.
    ///
    /// Example: `["geology","landform"]`
    pub fn loaded_layers(&self) -> String {
        let ids: Vec<&str> = self.layers.keys().map(String::as_str).collect();
        serde_json::to_string(&ids).expect("INVARIANT: Vec<&str> serialization is infallible")
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
    ///   "facilities": { "schools": 12, "medical": 28, "stations_nearby": 3 },
    ///   "zoning_distribution": [
    ///     { "zone": "商業地域", "ratio": 0.35 },
    ///     { "zone": "住居地域", "ratio": 0.45 }
    ///   ]
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
        let bbox =
            BBox::new(south, west, north, east).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.compute_stats_inner(&bbox)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Compute TLS for the given bounding box with specified weight preset.
    #[wasm_bindgen]
    pub fn compute_tls(
        &self,
        south: f64,
        west: f64,
        north: f64,
        east: f64,
        preset: &str,
    ) -> Result<String, JsValue> {
        let bbox =
            BBox::new(south, west, north, east).map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.compute_tls_inner(&bbox, preset)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

impl SpatialEngine {
    /// Internal load implementation returning `Result<_, WasmError>` for testability
    /// without `JsValue` (which panics on non-wasm32 targets).
    pub(crate) fn load_layer_inner(
        &mut self,
        layer_id: &str,
        fgb_bytes: &[u8],
    ) -> Result<u32, WasmError> {
        let features = parse_fgb(fgb_bytes)?;
        let count = u32::try_from(features.len()).expect("INVARIANT: feature count fits in u32");
        let index = LayerIndex::from_parsed(features, layer_id);
        self.layers.insert(layer_id.to_string(), index);
        Ok(count)
    }

    /// Internal GeoJSON load implementation returning `Result<_, WasmError>` for testability.
    pub(crate) fn load_geojson_layer_inner(
        &mut self,
        layer_id: &str,
        geojson: &str,
    ) -> Result<u32, WasmError> {
        let features = fgb_reader::parse_geojson_feature_collection(geojson)?;
        let count = u32::try_from(features.len()).expect("INVARIANT: feature count fits in u32");
        let index = LayerIndex::from_parsed(features, layer_id);
        self.layers.insert(layer_id.to_string(), index);
        Ok(count)
    }

    /// Internal query implementation returning `Result<_, WasmError>`.
    pub(crate) fn query_inner(&self, bbox: &BBox, layer_id: &str) -> Result<String, WasmError> {
        let index = self
            .layers
            .get(layer_id)
            .ok_or_else(|| WasmError::LayerNotFound(layer_id.to_string()))?;

        let indices = index.query_bbox(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        Ok(index.get_features_geojson(&indices))
    }

    /// Internal query_layers implementation returning `Result<_, WasmError>`.
    pub(crate) fn query_layers_inner(
        &self,
        bbox: &BBox,
        layer_ids: &str,
    ) -> Result<String, WasmError> {
        let mut result: HashMap<&str, serde_json::Value> = HashMap::new();

        for raw_id in layer_ids
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
        {
            let canonical = constants::canonical_layer_id(raw_id);
            if let Some(index) = self.layers.get(canonical.as_str()) {
                let indices =
                    index.query_bbox(bbox.south(), bbox.west(), bbox.north(), bbox.east());
                result.insert(raw_id, index.get_features_as_value(&indices));
            }
        }

        Ok(serde_json::to_string(&result)?)
    }

    /// Internal compute_stats implementation returning `Result<_, WasmError>`.
    pub(crate) fn compute_stats_inner(&self, bbox: &BBox) -> Result<String, WasmError> {
        let stats = self.compute_area_stats(bbox);
        Ok(serde_json::to_string(&stats)?)
    }

    /// Internal compute_tls implementation returning `Result<_, WasmError>`.
    pub(crate) fn compute_tls_inner(&self, bbox: &BBox, preset: &str) -> Result<String, WasmError> {
        let stats = self.compute_area_stats(bbox);
        let weight_preset: tls::WeightPreset = preset
            .parse()
            .expect("INVARIANT: WeightPreset::FromStr is infallible");
        let result = tls::compute_tls(&stats, weight_preset, &tls::NormalizationParams::TOKYO);
        Ok(serde_json::to_string(&result)?)
    }

    /// Compute [`AreaStats`] for the given bounding box.
    ///
    /// All individual layer queries degrade gracefully to zero / empty values
    /// when a layer has not been loaded.
    pub(crate) fn compute_area_stats(&self, bbox: &BBox) -> AreaStats {
        let bbox_rect = Rect::new(
            Coord {
                x: bbox.west(),
                y: bbox.south(),
            },
            Coord {
                x: bbox.east(),
                y: bbox.north(),
            },
        );

        // --- Land price ---
        let land_price = self
            .layers
            .get(constants::LAYER_LANDPRICE)
            .map(|idx| {
                let indices = idx.query_bbox(bbox.south(), bbox.west(), bbox.north(), bbox.east());
                compute_land_price_stats(&idx.stats_data, &indices)
            })
            .unwrap_or_else(|| compute_land_price_stats(&LayerStatsData::None, &[]));

        // --- Flood risk ---
        let flood_area_ratio = self.find_layer_ratio(
            &bbox_rect,
            bbox,
            &[constants::LAYER_FLOOD_HISTORY, constants::LAYER_FLOOD],
        );

        // --- Steep slope risk ---
        let steep_slope_area_ratio = self.find_layer_ratio(
            &bbox_rect,
            bbox,
            &[
                constants::LAYER_STEEP_SLOPE,
                constants::LAYER_STEEP_SLOPE_ALT,
            ],
        );

        // --- Composite risk ---
        let composite_risk = (flood_area_ratio * constants::RISK_WEIGHT_FLOOD
            + steep_slope_area_ratio * constants::RISK_WEIGHT_STEEP)
            .clamp(0.0, 1.0);

        // --- Schools ---
        let schools = self
            .layers
            .get(constants::LAYER_SCHOOLS)
            .map(|idx| {
                u32::try_from(
                    idx.query_bbox(bbox.south(), bbox.west(), bbox.north(), bbox.east())
                        .len(),
                )
                .expect("INVARIANT: hit count fits in u32")
            })
            .unwrap_or(0);

        // --- Medical ---
        let medical = self
            .layers
            .get(constants::LAYER_MEDICAL)
            .map(|idx| {
                u32::try_from(
                    idx.query_bbox(bbox.south(), bbox.west(), bbox.north(), bbox.east())
                        .len(),
                )
                .expect("INVARIANT: hit count fits in u32")
            })
            .unwrap_or(0);

        // --- Stations nearby ---
        let stations_nearby = self
            .layers
            .get(constants::LAYER_RAILWAY)
            .or_else(|| self.layers.get(constants::LAYER_STATION))
            .map(|layer| {
                u32::try_from(
                    layer
                        .query_bbox(bbox.south(), bbox.west(), bbox.north(), bbox.east())
                        .len(),
                )
                .expect("INVARIANT: hit count fits in u32")
            })
            .unwrap_or(0);

        // --- Zoning distribution ---
        let zoning_distribution = self
            .layers
            .get(constants::LAYER_ZONING)
            .map(|idx| {
                let indices = idx.query_bbox(bbox.south(), bbox.west(), bbox.north(), bbox.east());
                compute_zoning_distribution(&bbox_rect, &idx.stats_data, &indices)
                    .into_iter()
                    .map(|(zone, ratio)| ZoningEntry { zone, ratio })
                    .collect()
            })
            .unwrap_or_default();

        AreaStats {
            land_price,
            risk: RiskStats {
                flood_area_ratio,
                steep_slope_area_ratio,
                composite_risk,
            },
            facilities: FacilityStats {
                schools,
                medical,
                stations_nearby,
            },
            zoning_distribution,
        }
    }

    /// Query the first matching layer from `candidates` and compute its area ratio.
    ///
    /// Returns `0.0` if none of the candidate layer ids are loaded.
    fn find_layer_ratio(&self, bbox_rect: &Rect<f64>, bbox: &BBox, candidates: &[&str]) -> f64 {
        for &layer_id in candidates {
            if let Some(idx) = self.layers.get(layer_id) {
                let indices = idx.query_bbox(bbox.south(), bbox.west(), bbox.north(), bbox.east());
                return compute_area_ratio(bbox_rect, &idx.stats_data, &indices);
            }
        }
        0.0
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

    fn tokyo_bbox() -> BBox {
        BBox::new(35.53, 139.57, 35.82, 139.92).expect("valid tokyo bbox")
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
            .query_inner(&tokyo_bbox(), "geology")
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

        let london = BBox::new(51.3, -0.5, 51.7, 0.3).expect("valid london bbox");
        let geojson = engine
            .query_inner(&london, "geology")
            .expect("query_inner should succeed");

        let parsed: serde_json::Value = serde_json::from_str(&geojson).unwrap();
        let features = parsed["features"].as_array().unwrap();
        assert!(features.is_empty(), "London bbox should return no features");
    }

    #[test]
    fn query_unknown_layer_returns_err() {
        let engine = SpatialEngine::new();
        let result = engine.query_inner(&tokyo_bbox(), "nonexistent");
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
            .query_layers_inner(&tokyo_bbox(), "geology,landform")
            .expect("query_layers_inner should succeed");

        let result: serde_json::Value = serde_json::from_str(&result_json).unwrap();
        assert!(
            result.get("geology").is_some(),
            "result should contain geology"
        );
        assert!(
            result.get("landform").is_some(),
            "result should contain landform"
        );
    }

    #[test]
    fn query_layers_omits_unloaded_layers() {
        let mut engine = SpatialEngine::new();
        engine
            .load_layer_inner("geology", &geology_bytes())
            .expect("geology load should succeed");

        let result_json = engine
            .query_layers_inner(&tokyo_bbox(), "geology,not_loaded")
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
            .query_layers_inner(&tokyo_bbox(), "geology")
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
            .compute_stats_inner(&tokyo_bbox())
            .expect("compute_stats_inner should succeed on empty engine");

        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(v["land_price"]["count"], 0);
        assert!(
            v["land_price"]["avg_per_sqm"].is_null(),
            "avg_per_sqm should be null when no data"
        );
        assert_eq!(v["risk"]["flood_area_ratio"], 0.0);
        assert_eq!(v["risk"]["steep_slope_area_ratio"], 0.0);
        assert_eq!(v["risk"]["composite_risk"], 0.0);
        assert_eq!(v["facilities"]["schools"], 0);
        assert_eq!(v["facilities"]["medical"], 0);
        assert!(
            v["zoning_distribution"].as_array().unwrap().is_empty(),
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
            .compute_stats_inner(&tokyo_bbox())
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
            .compute_stats_inner(&tokyo_bbox())
            .expect("compute_stats_inner should succeed");

        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Verify top-level keys
        for key in &["land_price", "risk", "facilities", "zoning_distribution"] {
            assert!(v.get(key).is_some(), "missing top-level key: {key}");
        }

        // Verify land_price sub-keys
        for key in &[
            "avg_per_sqm",
            "median_per_sqm",
            "min_per_sqm",
            "max_per_sqm",
            "count",
        ] {
            assert!(
                v["land_price"].get(key).is_some(),
                "missing land_price key: {key}"
            );
        }

        // Verify risk sub-keys
        for key in &[
            "flood_area_ratio",
            "steep_slope_area_ratio",
            "composite_risk",
        ] {
            assert!(v["risk"].get(key).is_some(), "missing risk key: {key}");
        }

        // Verify facilities sub-keys
        for key in &["schools", "medical", "stations_nearby"] {
            assert!(
                v["facilities"].get(key).is_some(),
                "missing facilities key: {key}"
            );
        }

        // Verify zoning_distribution is an array
        assert!(
            v["zoning_distribution"].is_array(),
            "zoning_distribution should be an array"
        );
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
            .compute_stats_inner(&tokyo_bbox())
            .expect("compute_stats_inner should succeed");

        let v: serde_json::Value = serde_json::from_str(&json).unwrap();

        let flood_ratio = v["risk"]["flood_area_ratio"].as_f64().unwrap();
        let composite = v["risk"]["composite_risk"].as_f64().unwrap();

        // Ratios must be in valid range
        assert!(
            (0.0..=1.0).contains(&flood_ratio),
            "flood_ratio out of range: {flood_ratio}"
        );
        assert!(
            (0.0..=1.0).contains(&composite),
            "composite out of range: {composite}"
        );
    }

    // -------------------------------------------------------------------------
    // SpatialEngine::load_geojson_layer_inner
    // -------------------------------------------------------------------------

    #[test]
    fn load_geojson_layer_parses_feature_collection() {
        let mut engine = SpatialEngine::new();
        let geojson = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "geometry": {"type": "Point", "coordinates": [139.7, 35.68]},
                    "properties": {"name": "test"}
                }
            ]
        }"#;
        let count = engine
            .load_geojson_layer_inner("testlayer", geojson)
            .expect("load_geojson_layer_inner should succeed");
        assert_eq!(count, 1);
        // feature_count normalises via canonical_layer_id; use the same canonical form.
        assert_eq!(engine.feature_count("testlayer"), 1);
    }

    #[test]
    fn load_geojson_layer_invalid_json_returns_error() {
        let mut engine = SpatialEngine::new();
        let result = engine.load_geojson_layer_inner("bad", "not json");
        assert!(result.is_err(), "invalid JSON should return Err");
    }

    #[test]
    fn load_geojson_layer_missing_features_returns_error() {
        let mut engine = SpatialEngine::new();
        let result = engine.load_geojson_layer_inner("bad", r#"{"type": "FeatureCollection"}"#);
        assert!(
            result.is_err(),
            "missing 'features' array should return Err"
        );
    }

    #[test]
    fn load_geojson_layer_replaces_existing() {
        let mut engine = SpatialEngine::new();
        let geojson = r#"{"type":"FeatureCollection","features":[
            {"type":"Feature","geometry":{"type":"Point","coordinates":[139.7,35.68]},"properties":{"name":"a"}}
        ]}"#;
        engine.load_geojson_layer_inner("test", geojson).unwrap();
        assert_eq!(engine.feature_count("test"), 1);

        // Reload with same layer_id → replaced
        let geojson2 = r#"{"type":"FeatureCollection","features":[
            {"type":"Feature","geometry":{"type":"Point","coordinates":[139.7,35.68]},"properties":{"name":"b"}},
            {"type":"Feature","geometry":{"type":"Point","coordinates":[139.8,35.69]},"properties":{"name":"c"}}
        ]}"#;
        engine.load_geojson_layer_inner("test", geojson2).unwrap();
        assert_eq!(engine.feature_count("test"), 2);
    }

    #[test]
    fn geojson_layer_participates_in_query() {
        let mut engine = SpatialEngine::new();
        let geojson = r#"{"type":"FeatureCollection","features":[
            {"type":"Feature","geometry":{"type":"Point","coordinates":[139.75,35.68]},"properties":{}}
        ]}"#;
        engine
            .load_geojson_layer_inner("test-layer", geojson)
            .unwrap();

        let bbox = BBox::new(35.6, 139.7, 35.7, 139.8).unwrap();
        let result = engine.query_inner(&bbox, "test-layer").unwrap();
        assert!(result.contains("FeatureCollection"));
        assert!(result.contains("139.75"));
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
            .compute_stats_inner(&tokyo_bbox())
            .expect("compute_stats_inner should succeed");

        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        let school_count = v["facilities"]["schools"].as_u64().unwrap();
        assert!(school_count > 0, "should count schools in Tokyo bbox");
    }
}

//! R-tree spatial index wrapping [`rstar`] for fast bounding-box queries.
//!
//! [`LayerIndex`] stores GeoJSON Feature strings in a `Vec` and indexes their
//! envelopes in an [`RTree`]. Queries return a GeoJSON `FeatureCollection`
//! string ready for MapLibre GL consumption.

use rstar::{AABB, PointDistance, RTree, RTreeObject};

use crate::constants;
use crate::fgb_reader::ParsedFeature;

/// Statistics data associated with a layer, extracted during loading.
///
/// The variant chosen depends on `layer_id` and determines what kind of
/// spatial computation can be performed on the layer's features.
pub(crate) enum LayerStatsData {
    /// No stats-relevant data available for this layer.
    None,
    /// Land price points: price-per-sqm values indexed by feature position.
    PricePoints(Vec<f64>),
    /// Flood or steep-slope risk polygons indexed by feature position.
    AreaPolygons(Vec<Option<geo::Geometry<f64>>>),
    /// Zoning polygons: `(zone_type, geometry)` indexed by feature position.
    ZoningPolygons(Vec<(String, Option<geo::Geometry<f64>>)>),
    /// Facility point layers — only the hit count matters, no geometry stored.
    PointCount,
}

/// An R-tree entry that maps a feature's bounding envelope to its index in
/// [`LayerIndex::features_json`].
struct IndexedFeature {
    /// Position in the features_json Vec.
    index: u32,
    /// Axis-aligned bounding box: `[longitude, latitude]`.
    envelope: AABB<[f64; 2]>,
}

impl RTreeObject for IndexedFeature {
    type Envelope = AABB<[f64; 2]>;

    fn envelope(&self) -> Self::Envelope {
        self.envelope
    }
}

impl PointDistance for IndexedFeature {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        self.envelope.distance_2(point)
    }
}

/// A spatial index for one named layer.
///
/// Build with [`LayerIndex::from_parsed`]; query with [`LayerIndex::query_bbox`]
/// + [`LayerIndex::get_features_geojson`].
pub(crate) struct LayerIndex {
    tree: RTree<IndexedFeature>,
    /// Parallel storage of GeoJSON Feature strings, indexed by [`IndexedFeature::index`].
    features_json: Vec<String>,
    /// Statistics data extracted at load time, keyed by feature index.
    pub(crate) stats_data: LayerStatsData,
}

impl LayerIndex {
    /// Build a [`LayerIndex`] from a parsed FlatGeobuf feature set.
    ///
    /// Uses [`RTree::bulk_load`] for O(n log n) construction, which is faster
    /// than repeated insertions when all features are known upfront.
    ///
    /// The `layer_id` determines which [`LayerStatsData`] variant is extracted.
    pub(crate) fn from_parsed(features: Vec<ParsedFeature>, layer_id: &str) -> Self {
        let mut features_json: Vec<String> = Vec::with_capacity(features.len());
        let stats_data = extract_stats_data(&features, layer_id);

        let entries: Vec<IndexedFeature> = features
            .into_iter()
            .enumerate()
            .map(|(i, f)| {
                features_json.push(f.geojson);
                IndexedFeature {
                    index: i as u32,
                    envelope: AABB::from_corners([f.min_x, f.min_y], [f.max_x, f.max_y]),
                }
            })
            .collect();

        Self {
            tree: RTree::bulk_load(entries),
            features_json,
            stats_data,
        }
    }

    /// Return the indices of features whose envelopes intersect `[west, south, east, north]`.
    ///
    /// Coordinates follow RFC 7946 (`[longitude, latitude]`).
    pub(crate) fn query_bbox(&self, south: f64, west: f64, north: f64, east: f64) -> Vec<u32> {
        let query_envelope = AABB::from_corners([west, south], [east, north]);
        self.tree
            .locate_in_envelope_intersecting(&query_envelope)
            .map(|f| f.index)
            .collect()
    }

    /// Assemble a GeoJSON `FeatureCollection` from the given feature indices.
    ///
    /// Returns `{"type":"FeatureCollection","features":[...]}`.
    /// Indices that are out of bounds are silently skipped.
    pub(crate) fn get_features_geojson(&self, indices: &[u32]) -> String {
        let capacity = indices.len() * constants::GEOJSON_FEATURE_BYTES_ESTIMATE;
        let mut out = String::with_capacity(capacity + 40);
        out.push_str(constants::FC_HEADER);

        let mut first = true;
        for &idx in indices {
            if let Some(geojson) = self.features_json.get(idx as usize) {
                if !first {
                    out.push(',');
                }
                out.push_str(geojson);
                first = false;
            }
        }

        out.push_str(constants::FC_FOOTER);
        out
    }

    /// Assemble a GeoJSON `FeatureCollection` as a `serde_json::Value`.
    ///
    /// Unlike [`get_features_geojson`] which returns a String, this returns
    /// a structured Value — avoiding double-serialization in `query_layers`.
    pub(crate) fn get_features_as_value(&self, indices: &[u32]) -> serde_json::Value {
        let features: Vec<serde_json::Value> = indices
            .iter()
            .filter_map(|&idx| {
                self.features_json
                    .get(idx as usize)
                    .and_then(|s| serde_json::from_str(s).ok())
            })
            .collect();

        serde_json::json!({
            "type": "FeatureCollection",
            "features": features,
        })
    }

    /// Total number of features stored in this index.
    pub(crate) fn feature_count(&self) -> u32 {
        self.features_json.len() as u32
    }
}

/// Extract the appropriate [`LayerStatsData`] variant from parsed features
/// based on the `layer_id`.
fn extract_stats_data(features: &[ParsedFeature], layer_id: &str) -> LayerStatsData {
    match layer_id {
        constants::LAYER_LANDPRICE => {
            let prices: Vec<f64> = features
                .iter()
                .map(|f| {
                    f.properties
                        .as_ref()
                        .and_then(|p| p.get(constants::PROP_PRICE_PER_SQM))
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0)
                })
                .collect();
            LayerStatsData::PricePoints(prices)
        }
        constants::LAYER_FLOOD_HISTORY
        | constants::LAYER_FLOOD
        | constants::LAYER_STEEP_SLOPE
        | constants::LAYER_STEEP_SLOPE_ALT => {
            let geoms: Vec<Option<geo::Geometry<f64>>> =
                features.iter().map(|f| f.geometry_geo.clone()).collect();
            LayerStatsData::AreaPolygons(geoms)
        }
        constants::LAYER_ZONING => {
            let pairs: Vec<(String, Option<geo::Geometry<f64>>)> = features
                .iter()
                .map(|f| {
                    let zone_type = f
                        .properties
                        .as_ref()
                        .and_then(|p| p.get(constants::PROP_ZONE_TYPE))
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    (zone_type, f.geometry_geo.clone())
                })
                .collect();
            LayerStatsData::ZoningPolygons(pairs)
        }
        constants::LAYER_SCHOOLS
        | constants::LAYER_MEDICAL
        | constants::LAYER_RAILWAY
        | constants::LAYER_STATION => LayerStatsData::PointCount,
        _ => LayerStatsData::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fgb_reader::parse_fgb;

    const FGB_PATH: &str = "../../data/fgb/13/geology.fgb";

    fn load_geology_index() -> LayerIndex {
        let bytes = std::fs::read(FGB_PATH).expect("geology.fgb should exist");
        let features = parse_fgb(&bytes).expect("parse_fgb should succeed");
        LayerIndex::from_parsed(features, "geology")
    }

    #[test]
    fn from_parsed_stores_all_features() {
        let index = load_geology_index();
        assert_eq!(index.feature_count(), 133);
    }

    #[test]
    fn query_bbox_tokyo_23ku_returns_features() {
        let index = load_geology_index();
        // Rough bounding box for Tokyo 23 wards
        let indices = index.query_bbox(35.53, 139.57, 35.82, 139.92);
        assert!(
            !indices.is_empty(),
            "query over Tokyo 23ku bbox should return at least one feature"
        );
    }

    #[test]
    fn query_bbox_outside_data_returns_empty() {
        let index = load_geology_index();
        // Bounding box for London — well outside Tokyo data
        let indices = index.query_bbox(51.3, -0.5, 51.7, 0.3);
        assert!(
            indices.is_empty(),
            "query outside data extent should return no features"
        );
    }

    #[test]
    fn get_features_geojson_valid_feature_collection() {
        let index = load_geology_index();
        let indices = index.query_bbox(35.53, 139.57, 35.82, 139.92);
        assert!(!indices.is_empty());

        let geojson = index.get_features_geojson(&indices);
        let parsed: serde_json::Value =
            serde_json::from_str(&geojson).expect("should be valid JSON");

        assert_eq!(parsed["type"], "FeatureCollection");
        let features = parsed["features"]
            .as_array()
            .expect("features must be array");
        assert_eq!(features.len(), indices.len());
    }

    #[test]
    fn get_features_geojson_empty_indices_returns_empty_collection() {
        let index = load_geology_index();
        let geojson = index.get_features_geojson(&[]);
        assert_eq!(geojson, r#"{"type":"FeatureCollection","features":[]}"#);
    }

    #[test]
    fn get_features_geojson_out_of_bounds_index_skipped() {
        let index = load_geology_index();
        // index 0 is valid, 99999 is not
        let geojson = index.get_features_geojson(&[0, 99999]);
        let parsed: serde_json::Value = serde_json::from_str(&geojson).unwrap();
        let features = parsed["features"].as_array().unwrap();
        assert_eq!(
            features.len(),
            1,
            "out-of-bounds index should be silently skipped"
        );
    }

    #[test]
    fn from_parsed_geology_has_none_stats_data() {
        let index = load_geology_index();
        assert!(
            matches!(index.stats_data, LayerStatsData::None),
            "geology layer should have None stats data"
        );
    }

    #[test]
    fn from_parsed_schools_has_point_count_stats_data() {
        let bytes = std::fs::read(FGB_PATH).expect("geology.fgb should exist");
        let features = parse_fgb(&bytes).expect("parse_fgb should succeed");
        let index = LayerIndex::from_parsed(features, "schools");
        assert!(
            matches!(index.stats_data, LayerStatsData::PointCount),
            "schools layer should have PointCount stats data"
        );
    }
}

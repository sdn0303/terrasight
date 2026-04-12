//! FlatGeobuf reader that parses binary FGB data into [`ParsedFeature`] records.
//!
//! Each [`ParsedFeature`] carries a pre-computed bounding envelope (min_x/y, max_x/y)
//! suitable for bulk-loading into an R-tree, plus the serialized GeoJSON `Feature`
//! string for zero-copy retrieval at query time.

use std::io::Cursor;

use crate::constants;
use crate::error::WasmError;

use flatgeobuf::{FallibleStreamingIterator, FgbReader};
use geo::{Coord, LineString, MultiPolygon, Point, Polygon};
use geozero::FeatureAccess;
use geozero::geojson::GeoJsonWriter;

/// A single feature extracted from a FlatGeobuf file.
///
/// Coordinates follow RFC 7946: `[longitude, latitude]`.
pub(crate) struct ParsedFeature {
    /// Western boundary (minimum longitude).
    pub(crate) min_x: f64,
    /// Southern boundary (minimum latitude).
    pub(crate) min_y: f64,
    /// Eastern boundary (maximum longitude).
    pub(crate) max_x: f64,
    /// Northern boundary (maximum latitude).
    pub(crate) max_y: f64,
    /// Complete GeoJSON `Feature` string, e.g. `{"type":"Feature","geometry":{...},"properties":{...}}`.
    pub(crate) geojson: String,
    /// Parsed `geo` geometry for spatial computations. `None` if parsing fails.
    pub(crate) geometry_geo: Option<geo::Geometry<f64>>,
    /// Feature properties extracted from the GeoJSON. `None` if parsing fails.
    pub(crate) properties: Option<serde_json::Map<String, serde_json::Value>>,
}

/// Parse raw FlatGeobuf bytes into a vector of [`ParsedFeature`] records.
///
/// This function:
/// 1. Opens the FGB stream with [`FgbReader`].
/// 2. Selects all features sequentially (no spatial pre-filter).
/// 3. Serialises each feature to a GeoJSON string via [`GeoJsonWriter`].
/// 4. Computes the bounding envelope by walking the GeoJSON coordinate arrays.
///
/// # Errors
///
/// Returns `Err(WasmError)` if the FGB header cannot be read, the feature stream
/// fails mid-way, or a feature cannot be serialised to GeoJSON.
pub(crate) fn parse_fgb(bytes: &[u8]) -> Result<Vec<ParsedFeature>, WasmError> {
    let mut cursor = Cursor::new(bytes);
    let reader = FgbReader::open(&mut cursor).map_err(|e| WasmError::FgbOpen(e.to_string()))?;

    let mut feature_iter = reader
        .select_all_seq()
        .map_err(|e| WasmError::FgbIteration(e.to_string()))?;

    let mut features: Vec<ParsedFeature> = Vec::new();

    while let Some(feature) = feature_iter
        .next()
        .map_err(|e| WasmError::FgbIteration(e.to_string()))?
    {
        let mut buf: Vec<u8> = Vec::new();
        let mut writer = GeoJsonWriter::new(&mut buf);

        // Always pass idx=0: each feature uses its own fresh GeoJsonWriter, so
        // the writer must not prepend a comma separator (which it does for idx > 0).
        feature
            .process(&mut writer, 0)
            .map_err(|e| WasmError::GeoJsonSerialise(e.to_string()))?;

        let geojson = String::from_utf8(buf).map_err(|e| WasmError::Utf8(e.to_string()))?;

        let (min_x, min_y, max_x, max_y) = extract_bbox(&geojson)?;

        // Best-effort extraction of geo geometry and properties from the GeoJSON string.
        // Parse failures are silently converted to None.
        let (geometry_geo, properties) = extract_geo_and_properties(&geojson);

        features.push(ParsedFeature {
            min_x,
            min_y,
            max_x,
            max_y,
            geojson,
            geometry_geo,
            properties,
        });
    }

    Ok(features)
}

/// Extract a `geo::Geometry` and a properties map from a GeoJSON Feature string.
///
/// Returns `(None, None)` on any parse failure — this is best-effort extraction.
fn extract_geo_and_properties(
    geojson: &str,
) -> (
    Option<geo::Geometry<f64>>,
    Option<serde_json::Map<String, serde_json::Value>>,
) {
    let value: serde_json::Value = match serde_json::from_str(geojson) {
        Ok(v) => v,
        Err(_) => return (None, None),
    };

    let properties = value
        .get(constants::GEOJSON_KEY_PROPERTIES)
        .and_then(|p| p.as_object())
        .cloned();

    let geometry = value.get(constants::GEOJSON_KEY_GEOMETRY);
    let geo_geom = geometry.and_then(json_to_geo_geometry);

    (geo_geom, properties)
}

/// Convert a GeoJSON geometry JSON value to a `geo::Geometry<f64>`.
///
/// Supports Point, LineString, Polygon, MultiPolygon. Returns `None` for
/// unsupported types or malformed input.
fn json_to_geo_geometry(geom: &serde_json::Value) -> Option<geo::Geometry<f64>> {
    let geom_type = geom.get(constants::GEOJSON_KEY_TYPE)?.as_str()?;
    let coords = geom.get(constants::GEOJSON_KEY_COORDINATES)?;

    match geom_type {
        constants::GEOJSON_TYPE_POINT => {
            let arr = coords.as_array()?;
            if arr.len() < constants::MIN_COORD_PAIR_LEN {
                return None;
            }
            let x = arr[0].as_f64()?;
            let y = arr[1].as_f64()?;
            Some(geo::Geometry::Point(Point::new(x, y)))
        }
        constants::GEOJSON_TYPE_LINE_STRING => {
            let ring = json_to_coord_vec(coords)?;
            Some(geo::Geometry::LineString(LineString::new(ring)))
        }
        constants::GEOJSON_TYPE_POLYGON => {
            let polygon = json_to_polygon(coords)?;
            Some(geo::Geometry::Polygon(polygon))
        }
        constants::GEOJSON_TYPE_MULTI_POLYGON => {
            let polys_arr = coords.as_array()?;
            let polys: Vec<Polygon<f64>> = polys_arr.iter().filter_map(json_to_polygon).collect();
            if polys.is_empty() {
                return None;
            }
            Some(geo::Geometry::MultiPolygon(MultiPolygon::new(polys)))
        }
        _ => None,
    }
}

/// Convert a GeoJSON Polygon coordinates array to `geo::Polygon<f64>`.
fn json_to_polygon(coords: &serde_json::Value) -> Option<Polygon<f64>> {
    let rings = coords.as_array()?;
    let mut rings_iter = rings.iter();

    let exterior_coords = json_to_coord_vec(rings_iter.next()?)?;
    let exterior = LineString::new(exterior_coords);

    let interiors: Vec<LineString<f64>> = rings_iter
        .filter_map(|ring| json_to_coord_vec(ring).map(LineString::new))
        .collect();

    Some(Polygon::new(exterior, interiors))
}

/// Convert a GeoJSON coordinate array (array of `[x, y, ...]`) to `Vec<Coord<f64>>`.
fn json_to_coord_vec(arr: &serde_json::Value) -> Option<Vec<Coord<f64>>> {
    let points = arr.as_array()?;
    let coords: Vec<Coord<f64>> = points
        .iter()
        .filter_map(|pt| {
            let pair = pt.as_array()?;
            if pair.len() < 2 {
                return None;
            }
            let x = pair[0].as_f64()?;
            let y = pair[1].as_f64()?;
            Some(Coord { x, y })
        })
        .collect();
    if coords.is_empty() {
        None
    } else {
        Some(coords)
    }
}

/// Walk the `"coordinates"` subtree of a GeoJSON `Feature` string and return
/// `(min_x, min_y, max_x, max_y)` (i.e. west, south, east, north).
///
/// Supports Point, MultiPoint, LineString, MultiLineString, Polygon,
/// MultiPolygon, and GeometryCollection.
///
/// # Errors
///
/// Returns `Err(WasmError)` if the string cannot be parsed as JSON or contains no
/// coordinate pairs.
fn extract_bbox(geojson: &str) -> Result<(f64, f64, f64, f64), WasmError> {
    let value: serde_json::Value = serde_json::from_str(geojson)?;

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut found = false;

    let geometry = value
        .get(constants::GEOJSON_KEY_GEOMETRY)
        .ok_or_else(|| WasmError::GeoJsonParse("missing 'geometry' key".into()))?;
    collect_coords(
        geometry, &mut min_x, &mut min_y, &mut max_x, &mut max_y, &mut found,
    );

    if found {
        Ok((min_x, min_y, max_x, max_y))
    } else {
        Err(WasmError::GeoJsonParse(
            "no coordinate pairs found in geometry".into(),
        ))
    }
}

/// Recursively collect coordinate values from a GeoJSON geometry node.
fn collect_coords(
    node: &serde_json::Value,
    min_x: &mut f64,
    min_y: &mut f64,
    max_x: &mut f64,
    max_y: &mut f64,
    found: &mut bool,
) {
    match node {
        serde_json::Value::Object(map) => {
            // GeometryCollection: iterate `"geometries"` array
            if let Some(geoms) = map.get(constants::GEOJSON_KEY_GEOMETRIES) {
                if let Some(arr) = geoms.as_array() {
                    for g in arr {
                        collect_coords(g, min_x, min_y, max_x, max_y, found);
                    }
                }
                return;
            }
            // Normal geometry: descend into `"coordinates"`
            if let Some(coords) = map.get(constants::GEOJSON_KEY_COORDINATES) {
                collect_coords(coords, min_x, min_y, max_x, max_y, found);
            }
        }
        serde_json::Value::Array(arr) => {
            // Detect a coordinate pair/triple: first two elements must be numbers
            if arr.len() >= constants::MIN_COORD_PAIR_LEN
                && let (Some(x), Some(y)) = (arr[0].as_f64(), arr[1].as_f64())
            {
                if x < *min_x {
                    *min_x = x;
                }
                if x > *max_x {
                    *max_x = x;
                }
                if y < *min_y {
                    *min_y = y;
                }
                if y > *max_y {
                    *max_y = y;
                }
                *found = true;
                return; // leaf node, no further recursion needed
            }
            // Nested arrays (rings, multi-geometries): recurse
            for item in arr {
                collect_coords(item, min_x, min_y, max_x, max_y, found);
            }
        }
        // Numbers, strings, booleans, null: ignore
        _ => {}
    }
}

/// Parse a GeoJSON FeatureCollection string into [`ParsedFeature`] items.
///
/// Used for API-fetched layers that need to participate in stats
/// computation but are not served as FlatGeobuf.
pub(crate) fn parse_geojson_feature_collection(
    geojson: &str,
) -> Result<Vec<ParsedFeature>, WasmError> {
    let value: serde_json::Value = serde_json::from_str(geojson)?;
    let features = value["features"]
        .as_array()
        .ok_or_else(|| WasmError::GeoJsonParse("missing 'features' array".into()))?;

    let mut parsed = Vec::with_capacity(features.len());
    for feature in features {
        let feature_str = serde_json::to_string(feature)?;
        let (min_x, min_y, max_x, max_y) = extract_bbox(&feature_str)?;
        let (geometry_geo, properties) = extract_geo_and_properties(&feature_str);
        parsed.push(ParsedFeature {
            min_x,
            min_y,
            max_x,
            max_y,
            geojson: feature_str,
            geometry_geo,
            properties,
        });
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FGB_PATH: &str = "../../data/fgb/13/geology.fgb";

    #[test]
    fn parse_geology_fgb_returns_expected_feature_count() {
        let bytes = std::fs::read(FGB_PATH).expect("geology.fgb should exist at data/fgb/13/");
        let features = parse_fgb(&bytes).expect("parse_fgb should succeed");
        assert_eq!(
            features.len(),
            133,
            "geology.fgb should contain exactly 133 features"
        );
    }

    #[test]
    fn parse_geology_fgb_features_have_valid_bbox() {
        let bytes = std::fs::read(FGB_PATH).expect("geology.fgb should exist");
        let features = parse_fgb(&bytes).expect("parse_fgb should succeed");

        for (i, f) in features.iter().enumerate() {
            assert!(
                f.min_x <= f.max_x,
                "feature {i}: min_x ({}) must be <= max_x ({})",
                f.min_x,
                f.max_x
            );
            assert!(
                f.min_y <= f.max_y,
                "feature {i}: min_y ({}) must be <= max_y ({})",
                f.min_y,
                f.max_y
            );
            // Tokyo prefecture includes Ogasawara Islands (lat ~24-27) and
            // extends to Minami-Torishima (lng ~153). Use loose bounds.
            assert!(
                f.min_x >= 136.0 && f.max_x <= 154.0,
                "feature {i}: longitude out of Tokyo prefecture range: [{}, {}]",
                f.min_x,
                f.max_x
            );
            assert!(
                f.min_y >= 20.0 && f.max_y <= 36.5,
                "feature {i}: latitude out of Tokyo prefecture range: [{}, {}]",
                f.min_y,
                f.max_y
            );
        }
    }

    #[test]
    fn parse_geology_fgb_features_contain_valid_geojson() {
        let bytes = std::fs::read(FGB_PATH).expect("geology.fgb should exist");
        let features = parse_fgb(&bytes).expect("parse_fgb should succeed");

        for (i, f) in features.iter().enumerate() {
            let v: serde_json::Value = serde_json::from_str(&f.geojson)
                .unwrap_or_else(|e| panic!("feature {i}: invalid JSON: {e}"));
            // geozero GeoJsonWriter produces {"type":"Feature", "properties":{...}, "geometry":{...}}
            assert!(
                v.get("geometry").is_some(),
                "feature {i}: GeoJSON lacks geometry key"
            );
        }
    }

    #[test]
    fn parse_empty_bytes_returns_error() {
        let result = parse_fgb(&[]);
        assert!(result.is_err(), "empty bytes should return Err");
    }

    #[test]
    fn extract_bbox_point() {
        let geojson = r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[139.75,35.68]},"properties":{}}"#;
        let (min_x, min_y, max_x, max_y) = extract_bbox(geojson).unwrap();
        assert!((min_x - 139.75).abs() < 1e-9);
        assert!((min_y - 35.68).abs() < 1e-9);
        assert!((max_x - 139.75).abs() < 1e-9);
        assert!((max_y - 35.68).abs() < 1e-9);
    }

    #[test]
    fn extract_bbox_polygon() {
        let geojson = r#"{"type":"Feature","geometry":{"type":"Polygon","coordinates":[[[139.0,35.0],[140.0,35.0],[140.0,36.0],[139.0,36.0],[139.0,35.0]]]},"properties":{}}"#;
        let (min_x, min_y, max_x, max_y) = extract_bbox(geojson).unwrap();
        assert!((min_x - 139.0).abs() < 1e-9);
        assert!((min_y - 35.0).abs() < 1e-9);
        assert!((max_x - 140.0).abs() < 1e-9);
        assert!((max_y - 36.0).abs() < 1e-9);
    }

    #[test]
    fn parse_fgb_features_have_properties_field() {
        let bytes = std::fs::read(FGB_PATH).expect("geology.fgb should exist");
        let features = parse_fgb(&bytes).expect("parse_fgb should succeed");
        // geology features should have properties extracted (even if empty map)
        let has_some = features.iter().any(|f| f.properties.is_some());
        assert!(
            has_some,
            "at least some features should have extracted properties"
        );
    }

    #[test]
    fn extract_geo_and_properties_point() {
        let geojson = r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[139.75,35.68]},"properties":{"name":"test"}}"#;
        let (geom, props) = extract_geo_and_properties(geojson);
        assert!(geom.is_some(), "point geometry should be extracted");
        assert!(props.is_some(), "properties should be extracted");
        assert_eq!(
            props.unwrap().get("name").and_then(|v| v.as_str()),
            Some("test")
        );
    }

    #[test]
    fn extract_geo_and_properties_polygon() {
        let geojson = r#"{"type":"Feature","geometry":{"type":"Polygon","coordinates":[[[139.0,35.0],[140.0,35.0],[140.0,36.0],[139.0,36.0],[139.0,35.0]]]},"properties":{}}"#;
        let (geom, _props) = extract_geo_and_properties(geojson);
        assert!(geom.is_some(), "polygon geometry should be extracted");
        assert!(
            matches!(geom, Some(geo::Geometry::Polygon(_))),
            "should be a Polygon variant"
        );
    }

    #[test]
    fn extract_geo_and_properties_invalid_json_returns_none() {
        let (geom, props) = extract_geo_and_properties("not json");
        assert!(geom.is_none());
        assert!(props.is_none());
    }
}

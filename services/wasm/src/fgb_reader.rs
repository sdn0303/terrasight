//! FlatGeobuf reader that parses binary FGB data into [`ParsedFeature`] records.
//!
//! Each [`ParsedFeature`] carries a pre-computed bounding envelope (min_x/y, max_x/y)
//! suitable for bulk-loading into an R-tree, plus the serialized GeoJSON `Feature`
//! string for zero-copy retrieval at query time.

use std::io::Cursor;

use flatgeobuf::{FallibleStreamingIterator, FgbReader};
use geozero::FeatureAccess;
use geozero::geojson::GeoJsonWriter;

/// A single feature extracted from a FlatGeobuf file.
///
/// Coordinates follow RFC 7946: `[longitude, latitude]`.
pub struct ParsedFeature {
    /// Western boundary (minimum longitude).
    pub min_x: f64,
    /// Southern boundary (minimum latitude).
    pub min_y: f64,
    /// Eastern boundary (maximum longitude).
    pub max_x: f64,
    /// Northern boundary (maximum latitude).
    pub max_y: f64,
    /// Complete GeoJSON `Feature` string, e.g. `{"type":"Feature","geometry":{...},"properties":{...}}`.
    pub geojson: String,
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
/// Returns `Err(String)` if the FGB header cannot be read, the feature stream
/// fails mid-way, or a feature cannot be serialised to GeoJSON.
pub fn parse_fgb(bytes: &[u8]) -> Result<Vec<ParsedFeature>, String> {
    let mut cursor = Cursor::new(bytes);
    let reader = FgbReader::open(&mut cursor).map_err(|e| format!("FGB open error: {e}"))?;

    let mut feature_iter = reader
        .select_all_seq()
        .map_err(|e| format!("FGB select_all_seq error: {e}"))?;

    let mut features: Vec<ParsedFeature> = Vec::new();

    while let Some(feature) = feature_iter
        .next()
        .map_err(|e| format!("FGB iteration error: {e}"))?
    {
        let mut buf: Vec<u8> = Vec::new();
        let mut writer = GeoJsonWriter::new(&mut buf);

        // Always pass idx=0: each feature uses its own fresh GeoJsonWriter, so
        // the writer must not prepend a comma separator (which it does for idx > 0).
        feature
            .process(&mut writer, 0)
            .map_err(|e| format!("GeoJSON serialisation error: {e}"))?;

        let geojson =
            String::from_utf8(buf).map_err(|e| format!("UTF-8 conversion error: {e}"))?;

        let (min_x, min_y, max_x, max_y) = extract_bbox(&geojson)?;

        features.push(ParsedFeature {
            min_x,
            min_y,
            max_x,
            max_y,
            geojson,
        });
    }

    Ok(features)
}

/// Walk the `"coordinates"` subtree of a GeoJSON `Feature` string and return
/// `(min_x, min_y, max_x, max_y)` (i.e. west, south, east, north).
///
/// Supports Point, MultiPoint, LineString, MultiLineString, Polygon,
/// MultiPolygon, and GeometryCollection.
///
/// # Errors
///
/// Returns `Err(String)` if the string cannot be parsed as JSON or contains no
/// coordinate pairs.
fn extract_bbox(geojson: &str) -> Result<(f64, f64, f64, f64), String> {
    let value: serde_json::Value =
        serde_json::from_str(geojson).map_err(|e| format!("JSON parse error: {e}"))?;

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut found = false;

    let geometry = value.get("geometry").ok_or("missing 'geometry' key")?;
    collect_coords(geometry, &mut min_x, &mut min_y, &mut max_x, &mut max_y, &mut found);

    if found {
        Ok((min_x, min_y, max_x, max_y))
    } else {
        Err("no coordinate pairs found in geometry".to_string())
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
            if let Some(geoms) = map.get("geometries") {
                if let Some(arr) = geoms.as_array() {
                    for g in arr {
                        collect_coords(g, min_x, min_y, max_x, max_y, found);
                    }
                }
                return;
            }
            // Normal geometry: descend into `"coordinates"`
            if let Some(coords) = map.get("coordinates") {
                collect_coords(coords, min_x, min_y, max_x, max_y, found);
            }
        }
        serde_json::Value::Array(arr) => {
            // Detect a coordinate pair/triple: first two elements must be numbers
            if arr.len() >= 2
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
        let geojson =
            r#"{"type":"Feature","geometry":{"type":"Point","coordinates":[139.75,35.68]},"properties":{}}"#;
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
}

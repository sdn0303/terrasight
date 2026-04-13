/// A GeoJSON Feature representation independent of domain types.
///
/// The API binary converts this to its domain `GeoFeature` type after
/// receiving the parsed geometry and properties from PostGIS.
#[derive(Debug, Clone)]
pub struct RawGeoFeature {
    /// The GeoJSON geometry type string (e.g. `"Point"`, `"Polygon"`).
    pub geo_type: String,
    /// The `coordinates` array from the GeoJSON geometry object.
    pub coordinates: serde_json::Value,
    /// Arbitrary feature properties.
    pub properties: serde_json::Value,
}

/// Parse PostGIS `ST_AsGeoJSON` output into a [`RawGeoFeature`].
///
/// Extracts `type` and `coordinates` from the GeoJSON geometry object
/// and combines them with the provided `properties`.
///
/// # Examples
///
/// ```
/// use realestate_db::geo::to_raw_geo_feature;
/// use serde_json::json;
///
/// let geojson = json!({"type": "Point", "coordinates": [139.76, 35.68]});
/// let props = json!({"id": 1});
/// let feature = to_raw_geo_feature(geojson, props);
/// assert_eq!(feature.geo_type, "Point");
/// ```
pub fn to_raw_geo_feature(
    geojson: serde_json::Value,
    properties: serde_json::Value,
) -> RawGeoFeature {
    let geo_type = geojson
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| {
            tracing::warn!("PostGIS ST_AsGeoJSON missing 'type' field, defaulting to Point");
            "Point"
        })
        .to_string();
    let coordinates = geojson
        .get("coordinates")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    RawGeoFeature {
        geo_type,
        coordinates,
        properties,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_point_geometry() {
        let geojson = json!({"type": "Point", "coordinates": [139.76, 35.68]});
        let props = json!({"id": 1, "name": "test"});
        let f = to_raw_geo_feature(geojson, props);
        assert_eq!(f.geo_type, "Point");
        assert_eq!(f.coordinates, json!([139.76, 35.68]));
        assert_eq!(f.properties["id"], 1);
    }

    #[test]
    fn parses_polygon_geometry() {
        let geojson = json!({"type": "Polygon", "coordinates": [[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 0.0]]]});
        let props = json!({"zone": "residential"});
        let f = to_raw_geo_feature(geojson, props);
        assert_eq!(f.geo_type, "Polygon");
    }

    #[test]
    fn defaults_to_point_when_type_missing() {
        let geojson = json!({"coordinates": [0.0, 0.0]});
        let f = to_raw_geo_feature(geojson, json!({}));
        assert_eq!(f.geo_type, "Point");
    }

    #[test]
    fn handles_null_coordinates() {
        let geojson = json!({"type": "Point"});
        let f = to_raw_geo_feature(geojson, json!({}));
        assert!(f.coordinates.is_null());
    }
}

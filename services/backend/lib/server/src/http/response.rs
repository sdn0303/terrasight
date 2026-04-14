//! RFC 7946 GeoJSON response DTOs for MapLibre GL consumption.
//!
//! Provides [`FeatureCollectionDto`] and [`FeatureDto`] — the JSON shapes
//! returned by every spatial endpoint.  These are intentionally domain-free:
//! handlers convert domain types into these DTOs at the handler boundary.
//!
//! ## Coordinate order
//!
//! All coordinates must follow RFC 7946 ordering: **`[longitude, latitude]`**.
//! PostGIS returns coordinates in this order when the geometry SRID is 4326.
//!
//! ## Example
//!
//! ```rust
//! use terrasight_server::http::response::{FeatureCollectionDto, FeatureDto};
//! use serde_json::json;
//!
//! let fc = FeatureCollectionDto::new(vec![
//!     FeatureDto::new("Point".into(), json!([139.76, 35.68]), json!({"name": "Tokyo Station"})),
//! ]);
//! let json = serde_json::to_string(&fc).expect("serialization is infallible");
//! assert!(json.contains("FeatureCollection"));
//! ```

use serde::Serialize;

/// RFC 7946 GeoJSON FeatureCollection response.
///
/// Used by any handler that returns spatial data for MapLibre GL consumption.
/// Coordinates must follow RFC 7946 ordering: `[longitude, latitude]`.
///
/// # Example
///
/// ```rust
/// use terrasight_server::http::response::{FeatureCollectionDto, FeatureDto};
/// use serde_json::json;
///
/// let fc = FeatureCollectionDto::new(vec![
///     FeatureDto::new("Point".into(), json!([139.76, 35.68]), json!({"id": 1})),
/// ]);
/// assert_eq!(fc.r#type, "FeatureCollection");
/// ```
#[derive(Debug, Serialize)]
pub struct FeatureCollectionDto {
    /// The RFC 7946 type: always `"FeatureCollection"`.
    pub r#type: &'static str,
    /// Array of Feature objects.
    pub features: Vec<FeatureDto>,
}

/// A single GeoJSON Feature.
#[derive(Debug, Serialize)]
pub struct FeatureDto {
    /// The RFC 7946 type: always `"Feature"`.
    pub r#type: &'static str,
    /// Geometry object containing the spatial shape and coordinates.
    pub geometry: GeometryDto,
    /// Free-form properties object describing the feature.
    pub properties: serde_json::Value,
}

/// GeoJSON geometry object.
#[derive(Debug, Serialize)]
pub struct GeometryDto {
    /// The geometry type: `"Point"`, `"Polygon"`, `"LineString"`, etc.
    pub r#type: String,
    /// The coordinates in RFC 7946 order: `[longitude, latitude]` for points, nested arrays for other types.
    pub coordinates: serde_json::Value,
}

impl FeatureDto {
    /// Create a new Feature from raw GeoJSON components.
    ///
    /// Domain-independent: accepts raw type string and JSON values
    /// instead of domain-specific types.
    ///
    /// # Example
    ///
    /// ```rust
    /// use terrasight_server::http::response::FeatureDto;
    /// use serde_json::json;
    ///
    /// let f = FeatureDto::new("Point".into(), json!([139.76, 35.68]), json!({"name": "Tokyo"}));
    /// assert_eq!(f.r#type, "Feature");
    /// assert_eq!(f.geometry.r#type, "Point");
    /// ```
    pub fn new(
        geo_type: String,
        coordinates: serde_json::Value,
        properties: serde_json::Value,
    ) -> Self {
        Self {
            r#type: "Feature",
            geometry: GeometryDto {
                r#type: geo_type,
                coordinates,
            },
            properties,
        }
    }
}

impl FeatureCollectionDto {
    /// Create a FeatureCollection from a vector of Features.
    ///
    /// # Example
    ///
    /// ```rust
    /// use terrasight_server::http::response::{FeatureCollectionDto, FeatureDto};
    /// use serde_json::json;
    ///
    /// let fc = FeatureCollectionDto::new(vec![]);
    /// assert_eq!(fc.features.len(), 0);
    /// ```
    pub fn new(features: Vec<FeatureDto>) -> Self {
        Self {
            r#type: "FeatureCollection",
            features,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn feature_collection_serializes_correctly() {
        let fc = FeatureCollectionDto::new(vec![FeatureDto::new(
            "Point".into(),
            json!([139.76, 35.68]),
            json!({"id": 1}),
        )]);

        let json_val = serde_json::to_value(&fc).expect("serialization is infallible");
        assert_eq!(json_val["type"], "FeatureCollection");
        assert_eq!(json_val["features"].as_array().expect("array").len(), 1);
        assert_eq!(json_val["features"][0]["type"], "Feature");
        assert_eq!(json_val["features"][0]["geometry"]["type"], "Point");
        assert_eq!(json_val["features"][0]["properties"]["id"], 1);
    }

    #[test]
    fn empty_feature_collection() {
        let fc = FeatureCollectionDto::new(vec![]);
        let json_val = serde_json::to_value(&fc).expect("serialization is infallible");
        assert_eq!(json_val["features"].as_array().expect("array").len(), 0);
    }

    #[test]
    fn feature_dto_new_sets_type_to_feature() {
        let f = FeatureDto::new("Polygon".into(), json!([]), json!({}));
        assert_eq!(f.r#type, "Feature");
        assert_eq!(f.geometry.r#type, "Polygon");
    }

    #[test]
    fn geometry_dto_stores_coordinates() {
        let coords = json!([[139.0, 35.0], [140.0, 36.0]]);
        let f = FeatureDto::new("LineString".into(), coords.clone(), json!({}));
        assert_eq!(f.geometry.coordinates, coords);
    }
}

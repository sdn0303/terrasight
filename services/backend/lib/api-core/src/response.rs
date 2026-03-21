use serde::Serialize;

/// RFC 7946 GeoJSON FeatureCollection response.
///
/// Used by any handler that returns spatial data for MapLibre GL consumption.
/// Coordinates must follow RFC 7946 ordering: `[longitude, latitude]`.
///
/// # Example
///
/// ```rust
/// use realestate_api_core::response::{FeatureCollectionDto, FeatureDto};
/// use serde_json::json;
///
/// let fc = FeatureCollectionDto::new(vec![
///     FeatureDto::new("Point".into(), json!([139.76, 35.68]), json!({"id": 1})),
/// ]);
/// assert_eq!(fc.r#type, "FeatureCollection");
/// ```
#[derive(Debug, Serialize)]
pub struct FeatureCollectionDto {
    pub r#type: &'static str,
    pub features: Vec<FeatureDto>,
}

/// A single GeoJSON Feature.
#[derive(Debug, Serialize)]
pub struct FeatureDto {
    pub r#type: &'static str,
    pub geometry: GeometryDto,
    pub properties: serde_json::Value,
}

/// GeoJSON geometry object.
#[derive(Debug, Serialize)]
pub struct GeometryDto {
    pub r#type: String,
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
    /// use realestate_api_core::response::FeatureDto;
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
    /// use realestate_api_core::response::{FeatureCollectionDto, FeatureDto};
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

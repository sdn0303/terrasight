//! Shared response types for reinfolib API endpoints.
//!
//! These types cover the GeoJSON wrapper returned by all tile-based endpoints.
//! Individual endpoint properties are left as `serde_json::Value` because the
//! API surface has 50+ fields per endpoint and most callers consume the raw JSON.

use serde::{Deserialize, Serialize};

/// GeoJSON `FeatureCollection` wrapper for reinfolib tile API responses.
///
/// All tile-based endpoints (`XPT001`, `XPT002`, `XKT002`, etc.) return this
/// structure when `response_format=geojson`.
///
/// # Example
///
/// ```json
/// {
///   "type": "FeatureCollection",
///   "features": [
///     {
///       "type": "Feature",
///       "geometry": { "type": "Point", "coordinates": [139.76, 35.68] },
///       "properties": { "point_id": "L01-001" }
///     }
///   ]
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
pub struct GeoJsonResponse {
    /// Always `"FeatureCollection"`.
    #[serde(rename = "type")]
    pub r#type: String,
    /// Zero or more features.  Empty when no data exists for the requested tile.
    #[serde(default)]
    pub features: Vec<GeoJsonFeature>,
}

/// A single GeoJSON `Feature` within a [`GeoJsonResponse`].
#[derive(Debug, Serialize, Deserialize)]
pub struct GeoJsonFeature {
    /// Always `"Feature"`.
    #[serde(rename = "type")]
    pub r#type: String,
    /// GeoJSON geometry object (Point, Polygon, MultiPolygon, …).
    pub geometry: serde_json::Value,
    /// Endpoint-specific property bag.  `null` fields are normalised to
    /// `serde_json::Value::Null` via `#[serde(default)]`.
    #[serde(default)]
    pub properties: serde_json::Value,
}

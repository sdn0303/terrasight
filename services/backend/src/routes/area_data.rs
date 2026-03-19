use axum::Json;
use serde_json::{json, Value};

/// Stub handler for `GET /api/area-data`.
///
/// Returns geospatial layer data (GeoJSON FeatureCollections) for the given bbox.
/// Full implementation pending PostGIS query integration.
pub async fn get_area_data() -> Json<Value> {
    Json(json!({ "status": "not_implemented" }))
}

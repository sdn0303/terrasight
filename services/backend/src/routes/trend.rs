use axum::Json;
use serde_json::{json, Value};

/// Stub handler for `GET /api/trend`.
///
/// Returns land price trend data for the nearest observation point to the given coordinate.
/// Full implementation pending.
pub async fn get_trend() -> Json<Value> {
    Json(json!({ "status": "not_implemented" }))
}

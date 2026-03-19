use axum::Json;
use serde_json::{json, Value};

/// Stub handler for `GET /api/stats`.
///
/// Returns aggregated area statistics (land price distribution, risk ratios, facility counts)
/// for the given bbox. Full implementation pending.
pub async fn get_stats() -> Json<Value> {
    Json(json!({ "status": "not_implemented" }))
}

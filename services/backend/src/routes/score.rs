use axum::Json;
use serde_json::{json, Value};

/// Stub handler for `GET /api/score`.
///
/// Computes an investment score (0-100) from trend, risk, access, and yield components.
/// Full scoring logic pending in `services/scoring.rs`.
pub async fn get_score() -> Json<Value> {
    Json(json!({ "status": "not_implemented" }))
}

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::CoordQuery;
use crate::handler::response::TlsResponse;
use crate::usecase::compute_tls::ComputeTlsUsecase;

/// `GET /api/score?lat=35.68&lng=139.76`
///
/// Computes a Total Location Score (0–100) from five axes:
/// disaster risk, terrain quality, livability, future potential, and price value.
#[tracing::instrument(skip(usecase), fields(endpoint = "score"))]
pub async fn get_score(
    State(usecase): State<Arc<ComputeTlsUsecase>>,
    Query(params): Query<CoordQuery>,
) -> Result<Json<TlsResponse>, AppError> {
    let preset = params.parse_preset();
    let coord = params.into_domain().inspect(|c| {
        tracing::debug!(
            lat = c.lat(),
            lng = c.lng(),
            preset = ?preset,
            "TLS score request parsed"
        )
    })?;

    usecase
        .execute(&coord, preset)
        .await
        .inspect(|output| {
            tracing::info!(
                score = output.score,
                grade = output.grade.as_str(),
                "TLS score computed"
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "TLS score failed"))
        .map(|output| TlsResponse::new(coord.lat(), coord.lng(), output))
        .map(Json)
        .map_err(Into::into)
}

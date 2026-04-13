//! `GET /api/v1/score` handler.
//!
//! Computes the Total Location Score (TLS) for a geographic coordinate.
//! Delegates to [`ComputeTlsUsecase`] which aggregates data from multiple
//! domain repositories and applies the five-axis scoring formula.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::CoordQuery;
use crate::handler::response::TlsResponse;
use crate::usecase::compute_tls::ComputeTlsUsecase;

/// Handles `GET /api/v1/score`.
///
/// Computes the Total Location Score (0–100) for the supplied coordinate.
/// The score is composed of five weighted axes: disaster risk, terrain
/// quality, livability, future potential, and price value. The `preset`
/// query parameter selects the weight distribution (defaults to
/// `"balance"`; see [`WeightPreset`](terrasight_domain::scoring::tls::WeightPreset)).
///
/// # Errors
///
/// - [`AppError`] with `400 Bad Request` when `lat` or `lng` is outside
///   the valid WGS-84 range.
/// - [`AppError`] with `503 Service Unavailable` on a database error.
#[tracing::instrument(skip(usecase), fields(endpoint = "score"))]
pub(crate) async fn get_score(
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

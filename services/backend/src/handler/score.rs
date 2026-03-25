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
    let coord = params.into_domain()?;
    tracing::debug!(
        lat = coord.lat(),
        lng = coord.lng(),
        "TLS score request parsed"
    );
    let output = usecase.execute(&coord).await?;
    tracing::info!(
        score = output.score,
        grade = output.grade.as_str(),
        "TLS score computed"
    );
    Ok(Json(TlsResponse::from(output)))
}

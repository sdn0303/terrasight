use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::CoordQuery;
use crate::handler::response::ScoreResponse;
use crate::usecase::compute_score::ComputeScoreUsecase;

/// `GET /api/score?lat=35.68&lng=139.76`
///
/// Computes an investment score (0-100) from trend, risk, access, and yield components.
#[tracing::instrument(skip(usecase), fields(endpoint = "score"))]
pub async fn get_score(
    State(usecase): State<Arc<ComputeScoreUsecase>>,
    Query(params): Query<CoordQuery>,
) -> Result<Json<ScoreResponse>, AppError> {
    let coord = params.into_domain()?;
    tracing::debug!(lat = coord.lat(), lng = coord.lng(), "score request parsed");
    let score = usecase.execute(&coord).await?;
    tracing::info!(score = score.total(), "score computed");
    Ok(Json(ScoreResponse::from(score)))
}

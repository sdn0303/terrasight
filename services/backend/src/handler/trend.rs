use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::TrendQuery;
use crate::handler::response::TrendResponse;
use crate::usecase::get_trend::GetTrendUsecase;

/// `GET /api/trend?lat=35.68&lng=139.76&years=5`
///
/// Returns land price trend data for the nearest observation point.
#[tracing::instrument(skip(usecase), fields(endpoint = "trend"))]
pub async fn get_trend(
    State(usecase): State<Arc<GetTrendUsecase>>,
    Query(params): Query<TrendQuery>,
) -> Result<Json<TrendResponse>, AppError> {
    let (coord, years) = params.into_domain()?;
    tracing::debug!(
        lat = coord.lat(),
        lng = coord.lng(),
        years = years.value(),
        "trend request parsed"
    );
    let trend = usecase.execute(coord, years).await?;
    tracing::info!(
        cagr = trend.cagr,
        direction = trend.direction.as_str(),
        data_points = trend.data.len(),
        "trend response ready"
    );
    Ok(Json(TrendResponse::from(trend)))
}

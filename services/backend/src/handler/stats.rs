use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::BBoxQuery;
use crate::handler::response::StatsResponse;
use crate::usecase::get_stats::GetStatsUsecase;

/// `GET /api/stats?south=&west=&north=&east=`
///
/// Returns aggregated area statistics for the given bounding box.
#[tracing::instrument(skip(usecase), fields(endpoint = "stats"))]
pub async fn get_stats(
    State(usecase): State<Arc<GetStatsUsecase>>,
    Query(params): Query<BBoxQuery>,
) -> Result<Json<StatsResponse>, AppError> {
    let bbox = params.into_domain()?;
    tracing::debug!(
        south = bbox.south(),
        west = bbox.west(),
        north = bbox.north(),
        east = bbox.east(),
        "stats request parsed"
    );
    let stats = usecase.execute(&bbox).await?;
    let composite_risk_fmt = format!("{:.3}", stats.risk.composite_risk);
    tracing::info!(
        land_price_count = stats.land_price.count,
        composite_risk = %composite_risk_fmt,
        "stats response ready"
    );
    Ok(Json(StatsResponse::from(stats)))
}

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
    let (bbox, pref_code) = params.into_domain().inspect(|(b, _)| {
        tracing::debug!(
            south = b.south(),
            west = b.west(),
            north = b.north(),
            east = b.east(),
            "stats request parsed"
        )
    })?;

    usecase
        .execute(&bbox, pref_code.as_ref())
        .await
        .inspect(|stats| {
            tracing::info!(
                land_price_count = stats.land_price.count,
                composite_risk = stats.risk.composite_risk,
                "stats response ready"
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "stats lookup failed"))
        .map(StatsResponse::from)
        .map(Json)
        .map_err(Into::into)
}

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::AreaStatsQuery;
use crate::handler::response::AreaStatsResponse;
use crate::usecase::get_area_stats::GetAreaStatsUsecase;

/// `GET /api/area-stats?code={admin_code}`
///
/// Returns aggregated statistics for the given administrative area.
/// `code` is a 2-digit prefecture code (e.g. `"13"`) or 5-digit municipality
/// code (e.g. `"13105"` for Bunkyo-ku, Tokyo).
#[tracing::instrument(skip(usecase), fields(endpoint = "area-stats"))]
pub async fn get_area_stats(
    State(usecase): State<Arc<GetAreaStatsUsecase>>,
    Query(params): Query<AreaStatsQuery>,
) -> Result<Json<AreaStatsResponse>, AppError> {
    let code = params.into_domain()?;
    let stats = usecase.execute(&code).await?;
    tracing::info!(
        code = %stats.code,
        level = %stats.level,
        land_price_count = stats.land_price.count,
        "area-stats response ready"
    );
    Ok(Json(AreaStatsResponse::from(stats)))
}

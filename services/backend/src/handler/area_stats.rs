//! `GET /api/v1/area-stats` handler.
//!
//! Returns land price, disaster risk, and facility statistics aggregated
//! for a single administrative area — either a prefecture (2-digit code)
//! or a municipality (5-digit code). Delegates to [`GetAreaStatsUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::AreaStatsQuery;
use crate::handler::response::AreaStatsResponse;
use crate::usecase::get_area_stats::GetAreaStatsUsecase;

/// Handles `GET /api/v1/area-stats`.
///
/// Returns aggregated statistics for the given administrative area.
/// The `code` query parameter accepts a 2-digit prefecture code
/// (e.g. `"13"` for Tokyo) or a 5-digit municipality code
/// (e.g. `"13105"` for Bunkyo-ku).
///
/// # Errors
///
/// - [`AppError`] with `400 Bad Request` when `code` cannot be parsed as a
///   valid [`AreaCode`](crate::domain::value_object::AreaCode).
/// - [`AppError`] with `404 Not Found` when the area has no data.
/// - [`AppError`] with `503 Service Unavailable` on a database error.
#[tracing::instrument(skip(usecase), fields(endpoint = "area-stats"))]
pub(crate) async fn get_area_stats(
    State(usecase): State<Arc<GetAreaStatsUsecase>>,
    Query(params): Query<AreaStatsQuery>,
) -> Result<Json<AreaStatsResponse>, AppError> {
    let code = params
        .into_domain()
        .inspect(|c| tracing::debug!(code = c.as_str(), "area-stats request parsed"))?;

    usecase
        .execute(&code)
        .await
        .inspect(|stats| {
            tracing::info!(
                code = stats.code.as_str(),
                level = %stats.level,
                land_price_count = stats.land_price.count,
                "area-stats response ready"
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "area-stats lookup failed"))
        .map(AreaStatsResponse::from)
        .map(Json)
        .map_err(Into::into)
}

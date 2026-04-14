//! `GET /api/v1/stats` handler.
//!
//! Returns aggregated statistics (land price, disaster risk, facility counts,
//! zoning distribution) for any arbitrary bounding box. Delegates to
//! [`GetStatsUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::BBoxQuery;
use crate::handler::response::StatsResponse;
use crate::usecase::get_stats::GetStatsUsecase;

/// Handles `GET /api/v1/stats`.
///
/// Accepts four individual coordinate query parameters (`south`, `west`,
/// `north`, `east`) and an optional `pref_code`. Returns a [`StatsResponse`]
/// containing land price aggregates, composite disaster risk scores,
/// facility counts, and a zoning type distribution map.
///
/// # Errors
///
/// - [`AppError`] with `400 Bad Request` when any coordinate is out of range
///   or the bounding box area exceeds the configured maximum.
/// - [`AppError`] with `503 Service Unavailable` on a database error.
#[tracing::instrument(skip(usecase), fields(endpoint = "stats"))]
pub(crate) async fn get_stats(
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

//! `GET /api/v1/land-prices/aggregation` handler.
//!
//! Returns land price polygon aggregation as a GeoJSON FeatureCollection.
//! Delegates to [`GetLandPriceAggregationUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::BBoxQuery;
use crate::usecase::get_land_price_aggregation::GetLandPriceAggregationUsecase;

/// Handles `GET /api/v1/land-prices/aggregation`.
///
/// Query parameters: `south`, `west`, `north`, `east` (bbox), optional `pref_code`.
///
/// Returns a GeoJSON FeatureCollection with municipality polygon features
/// containing land price statistics (`avg_price`, `median_price`, `min_price`,
/// `max_price`, `count`, `prev_year_avg`, `change_pct`).
///
/// # Errors
///
/// - `400 Bad Request` — invalid bbox coordinates or prefecture code
/// - `408 Request Timeout` — aggregation query exceeded deadline
/// - `503 Service Unavailable` — database error
#[tracing::instrument(skip(usecase), fields(endpoint = "v1/land-prices/aggregation"))]
pub(crate) async fn get_land_price_aggregation(
    State(usecase): State<Arc<GetLandPriceAggregationUsecase>>,
    Query(params): Query<BBoxQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (bbox, pref_code) = params.into_domain().inspect(|(b, _)| {
        tracing::debug!(
            south = b.south(),
            west = b.west(),
            north = b.north(),
            east = b.east(),
            "land-price aggregation request parsed"
        );
    })?;

    usecase
        .execute(bbox, pref_code.as_ref())
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "land-price aggregation failed"))
        .map(Json)
        .map_err(Into::into)
}

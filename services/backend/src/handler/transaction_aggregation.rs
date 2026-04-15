//! `GET /api/v1/transactions/aggregation` handler.
//!
//! Returns transaction polygon aggregation as a GeoJSON FeatureCollection.
//! Delegates to [`GetTransactionAggregationUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::BBoxQuery;
use crate::usecase::get_transaction_aggregation::GetTransactionAggregationUsecase;

/// Handles `GET /api/v1/transactions/aggregation`.
///
/// Query parameters: `south`, `west`, `north`, `east` (bbox), optional `pref_code`.
///
/// Returns a GeoJSON FeatureCollection with municipality polygon features
/// containing transaction statistics (`tx_count`, `avg_price_sqm`,
/// `avg_total_price`).
///
/// # Errors
///
/// - `400 Bad Request` — invalid bbox coordinates or prefecture code
/// - `408 Request Timeout` — aggregation query exceeded deadline
/// - `503 Service Unavailable` — database error
#[tracing::instrument(skip(usecase), fields(endpoint = "v1/transactions/aggregation"))]
pub(crate) async fn get_transaction_aggregation(
    State(usecase): State<Arc<GetTransactionAggregationUsecase>>,
    Query(params): Query<BBoxQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let (bbox, pref_code) = params.into_domain().inspect(|(b, _)| {
        tracing::debug!(
            south = b.south(),
            west = b.west(),
            north = b.north(),
            east = b.east(),
            "transaction aggregation request parsed"
        );
    })?;

    usecase
        .execute(bbox, pref_code.as_ref())
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "transaction aggregation failed"))
        .map(Json)
        .map_err(Into::into)
}

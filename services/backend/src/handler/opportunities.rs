//! `GET /api/v1/opportunities` handler.
//!
//! Thin adapter: parse the query string into validated filters, delegate
//! to [`GetOpportunitiesUsecase`], and flatten the cached response into
//! an [`OpportunitiesResponseDto`] for JSON serialization.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::OpportunitiesQuery;
use crate::handler::response::OpportunitiesResponseDto;
use crate::usecase::get_opportunities::GetOpportunitiesUsecase;

/// `GET /api/v1/opportunities?bbox=sw_lng,sw_lat,ne_lng,ne_lat&limit=50&...`
///
/// Returns up to `OpportunityLimit::MAX` TLS-enriched land price records
/// within `bbox`. Subsequent calls with the same validated filter set
/// hit the 60-second in-memory cache.
///
/// # Errors
///
/// - `400 Bad Request` — unparseable `bbox`, unknown `risk_max`,
///   negative `price_min`/`price_max`, or any other query validation
///   failure.
/// - `408 Request Timeout` — the response exceeded the 8-second budget.
/// - `503 Service Unavailable` — database failure during the flat fetch.
#[tracing::instrument(skip(usecase), fields(endpoint = "v1/opportunities"))]
pub async fn get_opportunities(
    State(usecase): State<Arc<GetOpportunitiesUsecase>>,
    Query(params): Query<OpportunitiesQuery>,
) -> Result<Json<OpportunitiesResponseDto>, AppError> {
    let filters = params
        .into_filters()
        .inspect(|f| tracing::debug!(limit = f.limit.get(), "opportunities request parsed"))?;

    usecase
        .execute(filters)
        .await
        .inspect(|response| {
            tracing::info!(
                items = response.items.len(),
                total = response.total,
                truncated = response.truncated,
                "opportunities response ready",
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "opportunities usecase failed"))
        .map(OpportunitiesResponseDto::from)
        .map(Json)
        .map_err(Into::into)
}

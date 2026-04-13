//! `GET /api/v1/trend` handler.
//!
//! Returns a CAGR-annotated time series of land price observations for the
//! survey point nearest to the requested coordinate. Delegates to
//! [`GetTrendUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::TrendQuery;
use crate::handler::response::TrendResponse;
use crate::usecase::get_trend::GetTrendUsecase;

/// Handles `GET /api/v1/trend`.
///
/// Required query parameters: `lat` and `lng` (WGS-84 decimal degrees).
/// Optional `years` lookback window (defaults to
/// [`TREND_DEFAULT_YEARS`](crate::domain::constants::TREND_DEFAULT_YEARS)).
///
/// Returns a [`TrendResponse`] containing:
/// - `location` — address and distance to the nearest survey point
/// - `data` — sorted `(year, price_per_sqm)` time series
/// - `cagr` — compound annual growth rate over the lookback window
/// - `direction` — trend direction label (`"rising"`, `"stable"`, `"falling"`)
///
/// # Errors
///
/// - [`AppError`] with `400 Bad Request` when `lat` or `lng` is out of range.
/// - [`AppError`] with `404 Not Found` when no observation point exists near
///   the coordinate.
/// - [`AppError`] with `503 Service Unavailable` on a database error.
#[tracing::instrument(skip(usecase), fields(endpoint = "trend"))]
pub(crate) async fn get_trend(
    State(usecase): State<Arc<GetTrendUsecase>>,
    Query(params): Query<TrendQuery>,
) -> Result<Json<TrendResponse>, AppError> {
    let (coord, years) = params.into_domain().inspect(|(c, y)| {
        tracing::debug!(
            lat = c.lat(),
            lng = c.lng(),
            years = y.value(),
            "trend request parsed"
        )
    })?;

    usecase
        .execute(coord, years)
        .await
        .inspect(|trend| {
            tracing::info!(
                cagr = trend.cagr,
                direction = trend.direction.as_str(),
                data_points = trend.data.len(),
                "trend response ready"
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "trend lookup failed"))
        .map(TrendResponse::from)
        .map(Json)
        .map_err(Into::into)
}

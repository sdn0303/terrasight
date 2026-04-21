//! `GET /api/v1/population` handler.
//!
//! Returns municipality-level population data from `mv_population_summary`
//! for the requested prefecture. Delegates to [`GetPopulationUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::population::PopulationQuery;
use crate::handler::response::population::PopulationResponse;
use crate::usecase::get_population::GetPopulationUsecase;

/// Handles `GET /api/v1/population`.
///
/// Required query parameter: `pref_code` (2-digit prefecture code, e.g.
/// `"13"` for Tokyo).
///
/// Returns a JSON array of [`PopulationResponse`] objects ordered by
/// municipality code.
///
/// # Errors
///
/// - [`AppError`] with `400 Bad Request` when `pref_code` is invalid.
/// - [`AppError`] with `503 Service Unavailable` on a database error.
#[tracing::instrument(skip(usecase), fields(endpoint = "population"))]
pub async fn get_population(
    State(usecase): State<Arc<GetPopulationUsecase>>,
    Query(params): Query<PopulationQuery>,
) -> Result<Json<Vec<PopulationResponse>>, AppError> {
    let pref_code = params.into_domain()?;

    usecase
        .execute(&pref_code)
        .await
        .inspect(|items| tracing::info!(count = items.len(), "population response ready"))
        .inspect_err(|e| tracing::warn!(error = %e, "population lookup failed"))
        .map(|items| items.into_iter().map(PopulationResponse::from).collect())
        .map(Json)
        .map_err(Into::into)
}

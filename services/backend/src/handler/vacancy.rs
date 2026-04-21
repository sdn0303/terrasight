//! `GET /api/v1/vacancy` handler.
//!
//! Returns municipality-level housing vacancy data from `mv_vacancy_summary`
//! for the requested prefecture. Delegates to [`GetVacancyUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::vacancy::VacancyQuery;
use crate::handler::response::vacancy::VacancyResponse;
use crate::usecase::get_vacancy::GetVacancyUsecase;

/// Handles `GET /api/v1/vacancy`.
///
/// Required query parameter: `pref_code` (2-digit prefecture code, e.g.
/// `"13"` for Tokyo).
///
/// Returns a JSON array of [`VacancyResponse`] objects ordered by
/// municipality code.
///
/// # Errors
///
/// - [`AppError`] with `400 Bad Request` when `pref_code` is invalid.
/// - [`AppError`] with `503 Service Unavailable` on a database error.
#[tracing::instrument(skip(usecase), fields(endpoint = "vacancy"))]
pub async fn get_vacancy(
    State(usecase): State<Arc<GetVacancyUsecase>>,
    Query(params): Query<VacancyQuery>,
) -> Result<Json<Vec<VacancyResponse>>, AppError> {
    let pref_code = params.into_domain()?;

    usecase
        .execute(&pref_code)
        .await
        .inspect(|items| tracing::info!(count = items.len(), "vacancy response ready"))
        .inspect_err(|e| tracing::warn!(error = %e, "vacancy lookup failed"))
        .map(|items| items.into_iter().map(VacancyResponse::from).collect())
        .map(Json)
        .map_err(Into::into)
}

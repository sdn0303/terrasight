//! `GET /api/v1/municipalities` handler.
//!
//! Returns all municipality records for a given prefecture, used by the
//! frontend's city-code filter drop-down. Delegates to
//! [`GetMunicipalitiesUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::municipality::MunicipalitiesQuery;
use crate::handler::response::municipality::MunicipalityResponse;
use crate::usecase::get_municipalities::GetMunicipalitiesUsecase;

/// Handles `GET /api/v1/municipalities`.
///
/// Required query parameter: `pref_code` (2-digit prefecture code,
/// e.g. `"13"` for Tokyo).
///
/// Returns a JSON array of [`MunicipalityResponse`] objects, each
/// containing `city_code`, `city_name`, and `pref_code`.
///
/// # Errors
///
/// - [`AppError`] with `400 Bad Request` when `pref_code` fails
///   [`PrefCode`](crate::domain::value_object::PrefCode) validation.
/// - [`AppError`] with `503 Service Unavailable` on a database error.
#[tracing::instrument(skip(usecase), fields(endpoint = "municipalities"))]
pub async fn get_municipalities(
    State(usecase): State<Arc<GetMunicipalitiesUsecase>>,
    Query(params): Query<MunicipalitiesQuery>,
) -> Result<Json<Vec<MunicipalityResponse>>, AppError> {
    let pref_code = params.into_domain()?;

    usecase
        .execute(&pref_code)
        .await
        .inspect(|items| tracing::info!(count = items.len(), "municipalities response ready"))
        .inspect_err(|e| tracing::warn!(error = %e, "municipalities lookup failed"))
        .map(|items| items.into_iter().map(MunicipalityResponse::from).collect())
        .map(Json)
        .map_err(Into::into)
}

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::municipality::MunicipalitiesQuery;
use crate::handler::response::municipality::MunicipalityResponse;
use crate::usecase::get_municipalities::GetMunicipalitiesUsecase;

/// `GET /api/v1/municipalities?pref_code=`
///
/// Returns all municipalities for the given prefecture.
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

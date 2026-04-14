//! `GET /api/v1/appraisals` handler.
//!
//! Returns government appraisal (公示地価) records for a given prefecture,
//! optionally narrowed to a single municipality. Delegates to
//! [`GetAppraisalsUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::appraisal::AppraisalsQuery;
use crate::handler::response::appraisal::AppraisalDetailResponse;
use crate::usecase::get_appraisals::GetAppraisalsUsecase;

/// Handles `GET /api/v1/appraisals`.
///
/// Required query parameter: `pref_code` (2-digit prefecture code, e.g.
/// `"13"` for Tokyo). Optional `city_code` (5-digit municipality code)
/// narrows results to a single city; if supplied, it must belong to the
/// given `pref_code` or the request is rejected.
///
/// Returns a JSON array of [`AppraisalDetailResponse`] objects.
///
/// # Errors
///
/// - [`AppError`] with `400 Bad Request` when `pref_code` is invalid,
///   `city_code` is malformed, or `city_code` does not match `pref_code`.
/// - [`AppError`] with `503 Service Unavailable` on a database error.
#[tracing::instrument(skip(usecase), fields(endpoint = "appraisals"))]
pub async fn get_appraisals(
    State(usecase): State<Arc<GetAppraisalsUsecase>>,
    Query(params): Query<AppraisalsQuery>,
) -> Result<Json<Vec<AppraisalDetailResponse>>, AppError> {
    let (pref_code, city_code) = params.into_domain()?;

    usecase
        .execute(&pref_code, city_code.as_ref())
        .await
        .inspect(|items| tracing::info!(count = items.len(), "appraisals response ready"))
        .inspect_err(|e| tracing::warn!(error = %e, "appraisals lookup failed"))
        .map(|items| {
            items
                .into_iter()
                .map(AppraisalDetailResponse::from)
                .collect()
        })
        .map(Json)
        .map_err(Into::into)
}

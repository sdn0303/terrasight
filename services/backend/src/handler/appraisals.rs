use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::appraisal::AppraisalsQuery;
use crate::handler::response::appraisal::AppraisalDetailResponse;
use crate::usecase::get_appraisals::GetAppraisalsUsecase;

/// `GET /api/v1/appraisals?pref_code=&city_code=`
///
/// Returns appraisal records for the given prefecture, optionally filtered by city code.
#[tracing::instrument(skip(usecase), fields(endpoint = "appraisals"))]
pub async fn get_appraisals(
    State(usecase): State<Arc<GetAppraisalsUsecase>>,
    Query(params): Query<AppraisalsQuery>,
) -> Result<Json<Vec<AppraisalDetailResponse>>, AppError> {
    let (pref_code, city_code) = params.into_domain()?;

    usecase
        .execute(&pref_code, city_code.as_deref())
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

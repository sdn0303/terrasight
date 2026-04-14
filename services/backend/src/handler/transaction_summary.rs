//! `GET /api/v1/transactions/summary` handler.
//!
//! Returns pre-aggregated transaction statistics bucketed by city, year,
//! and property type for a prefecture. Used by the frontend's trend charts
//! and comparison tables. Delegates to [`GetTransactionSummaryUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::transaction::TransactionSummaryQuery;
use crate::handler::response::transaction::TransactionSummaryResponse;
use crate::usecase::get_transaction_summary::GetTransactionSummaryUsecase;

/// Handles `GET /api/v1/transactions/summary`.
///
/// Required query parameter: `pref_code` (2-digit prefecture code). Optional
/// `year_from` (integer) and `property_type` (raw Japanese string such as
/// `"宅地(土地)"`) further narrow the aggregation.
///
/// Returns a JSON array of [`TransactionSummaryResponse`] objects, each
/// representing one `(city_code, transaction_year, property_type)` bucket
/// with count, average, and median price metrics.
///
/// # Errors
///
/// - [`AppError`] with `400 Bad Request` when `pref_code` is invalid or
///   `year_from` is outside the valid range.
/// - [`AppError`] with `503 Service Unavailable` on a database error.
#[tracing::instrument(skip(usecase), fields(endpoint = "transactions/summary"))]
pub async fn get_transaction_summary(
    State(usecase): State<Arc<GetTransactionSummaryUsecase>>,
    Query(params): Query<TransactionSummaryQuery>,
) -> Result<Json<Vec<TransactionSummaryResponse>>, AppError> {
    let (pref, year, prop_type) = params.into_domain()?;
    usecase
        .execute(&pref, year.as_ref(), prop_type.as_deref())
        .await
        .map(|v| {
            v.into_iter()
                .map(TransactionSummaryResponse::from)
                .collect()
        })
        .map(Json)
        .map_err(Into::into)
}

//! `GET /api/v1/transactions` handler.
//!
//! Returns individual real estate transaction records for a city,
//! optionally filtered by year and capped with a limit. Delegates to
//! [`GetTransactionsUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::domain::value_object::{CityCode, Year};
use crate::handler::error::AppError;
use crate::handler::request::transaction::TransactionsQuery;
use crate::handler::response::transaction::TransactionDetailResponse;
use crate::usecase::get_transactions::GetTransactionsUsecase;

/// Handles `GET /api/v1/transactions`.
///
/// Required query parameter: `city_code` (5-digit municipality code, e.g.
/// `"13101"`). Optional `year_from` (integer survey year) and `limit`
/// (positive integer, clamped to `MAX_TRANSACTION_LIMIT` = 200 by the
/// usecase layer; defaults to `DEFAULT_TRANSACTION_LIMIT` = 50).
///
/// Returns a JSON array of [`TransactionDetailResponse`] objects.
///
/// # Errors
///
/// - [`AppError`] with `400 Bad Request` when `city_code` fails
///   [`CityCode`] validation or `year_from` is outside the valid range.
/// - [`AppError`] with `503 Service Unavailable` on a database error.
#[tracing::instrument(skip(usecase), fields(endpoint = "transactions"))]
pub async fn get_transactions(
    State(usecase): State<Arc<GetTransactionsUsecase>>,
    Query(params): Query<TransactionsQuery>,
) -> Result<Json<Vec<TransactionDetailResponse>>, AppError> {
    let city = CityCode::new(&params.city_code)?;
    let year = params.year_from.map(Year::new).transpose()?;
    usecase
        .execute(&city, year.as_ref(), params.limit)
        .await
        .map(|v| v.into_iter().map(TransactionDetailResponse::from).collect())
        .map(Json)
        .map_err(Into::into)
}

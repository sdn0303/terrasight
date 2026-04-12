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

/// `GET /api/v1/transactions?city_code=13101&year_from=2020&limit=50`
///
/// Returns individual transaction records for the given city code,
/// optionally filtered by year and limited in count.
#[tracing::instrument(skip(usecase), fields(endpoint = "transactions"))]
pub async fn get_transactions(
    State(usecase): State<Arc<GetTransactionsUsecase>>,
    Query(params): Query<TransactionsQuery>,
) -> Result<Json<Vec<TransactionDetailResponse>>, AppError> {
    let city = CityCode::new(&params.city_code)?;
    let year = params.year_from.map(Year::new).transpose()?;
    usecase
        .execute(city.as_str(), year.as_ref(), params.limit)
        .await
        .map(|v| v.into_iter().map(TransactionDetailResponse::from).collect())
        .map(Json)
        .map_err(Into::into)
}

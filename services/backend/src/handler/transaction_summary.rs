use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::transaction::TransactionSummaryQuery;
use crate::handler::response::transaction::TransactionSummaryResponse;
use crate::usecase::get_transaction_summary::GetTransactionSummaryUsecase;

/// `GET /api/v1/transactions/summary?pref_code=13&year_from=2020&property_type=…`
///
/// Returns aggregated transaction summaries (city / year / property_type buckets)
/// for the given prefecture, optionally filtered by year and property type.
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

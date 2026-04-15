//! Usecase: fetch transaction polygon aggregation as GeoJSON.
//!
//! Delegates to [`AggregationRepository::transaction_aggregation`] for typed
//! rows, then assembles the GeoJSON FeatureCollection. Called by
//! `GET /api/v1/transactions/aggregation`.

use std::sync::Arc;

use serde_json::json;

use crate::domain::error::DomainError;
use crate::domain::model::{BBox, PrefCode, TransactionAggRow};
use crate::domain::repository::AggregationRepository;

/// Usecase for `GET /api/v1/transactions/aggregation`.
pub(crate) struct GetTransactionAggregationUsecase {
    repo: Arc<dyn AggregationRepository>,
}

impl GetTransactionAggregationUsecase {
    /// Construct the usecase with the given aggregation repository.
    pub(crate) fn new(repo: Arc<dyn AggregationRepository>) -> Self {
        Self { repo }
    }

    /// Execute the aggregation query and return a GeoJSON FeatureCollection.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    #[tracing::instrument(skip(self), fields(usecase = "get_transaction_aggregation"))]
    pub(crate) async fn execute(
        &self,
        bbox: BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<serde_json::Value, DomainError> {
        let rows = self
            .repo
            .transaction_aggregation(&bbox, pref_code)
            .await
            .inspect(|rows| {
                tracing::info!(feature_count = rows.len(), "transaction aggregation ready");
            })?;

        Ok(to_feature_collection(rows))
    }
}

/// Assemble a GeoJSON FeatureCollection from typed aggregation rows.
///
/// Each row's `geometry` field is the raw `ST_AsGeoJSON` output (already
/// valid GeoJSON geometry), so it is passed through without re-parsing.
fn to_feature_collection(rows: Vec<TransactionAggRow>) -> serde_json::Value {
    let features: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            json!({
                "type": "Feature",
                "geometry": row.geometry,
                "properties": {
                    "admin_code": row.admin_code,
                    "city_name": row.city_name,
                    "tx_count": row.tx_count,
                    "avg_price_sqm": row.avg_price_sqm,
                    "avg_total_price": row.avg_total_price,
                },
            })
        })
        .collect();

    json!({
        "type": "FeatureCollection",
        "features": features,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_feature_collection_empty() {
        let fc = to_feature_collection(vec![]);
        assert_eq!(fc["type"], "FeatureCollection");
        assert_eq!(fc["features"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn to_feature_collection_builds_features() {
        let row = TransactionAggRow {
            admin_code: "13101".into(),
            city_name: "千代田区".into(),
            geometry: json!({"type": "MultiPolygon", "coordinates": []}),
            tx_count: 42,
            avg_price_sqm: 850000.0,
            avg_total_price: 42500000.0,
        };
        let fc = to_feature_collection(vec![row]);
        let features = fc["features"].as_array().unwrap();
        assert_eq!(features.len(), 1);
        let props = &features[0]["properties"];
        assert_eq!(props["tx_count"], 42);
        assert_eq!(props["admin_code"], "13101");
    }
}

//! Usecase: fetch land price polygon aggregation as GeoJSON.
//!
//! Delegates to [`AggregationRepository::land_price_aggregation`] for typed
//! rows, computes [`LandPriceAggRow::change_pct`] in the application layer,
//! then assembles the GeoJSON FeatureCollection. Called by
//! `GET /api/v1/land-prices/aggregation`.

use std::sync::Arc;

use serde_json::json;

use crate::domain::error::DomainError;
use crate::domain::model::{BBox, LandPriceAggRow, PrefCode};
use crate::domain::repository::AggregationRepository;

/// Usecase for `GET /api/v1/land-prices/aggregation`.
pub(crate) struct GetLandPriceAggregationUsecase {
    repo: Arc<dyn AggregationRepository>,
}

impl GetLandPriceAggregationUsecase {
    /// Construct the usecase with the given aggregation repository.
    pub(crate) fn new(repo: Arc<dyn AggregationRepository>) -> Self {
        Self { repo }
    }

    /// Execute the aggregation query and return a GeoJSON FeatureCollection.
    ///
    /// [`LandPriceAggRow::change_pct`] is computed here (application layer)
    /// rather than in SQL, keeping business logic out of the infra layer.
    ///
    /// # Errors
    ///
    /// Propagates [`DomainError`] from the repository.
    #[tracing::instrument(skip(self), fields(usecase = "get_land_price_aggregation"))]
    pub(crate) async fn execute(
        &self,
        bbox: BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<serde_json::Value, DomainError> {
        let rows = self
            .repo
            .land_price_aggregation(&bbox, pref_code)
            .await
            .inspect(|rows| {
                tracing::info!(feature_count = rows.len(), "land-price aggregation ready");
            })?;

        Ok(to_feature_collection(rows))
    }
}

/// Assemble a GeoJSON FeatureCollection from typed aggregation rows.
///
/// Each row's `geometry` field is the raw `ST_AsGeoJSON` output (already
/// valid GeoJSON geometry), so it is passed through without re-parsing.
fn to_feature_collection(rows: Vec<LandPriceAggRow>) -> serde_json::Value {
    let features: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            let change_pct = row.change_pct();
            json!({
                "type": "Feature",
                "geometry": row.geometry,
                "properties": {
                    "admin_code": row.admin_code,
                    "pref_name": row.pref_name,
                    "city_name": row.city_name,
                    "avg_price": row.avg_price,
                    "median_price": row.median_price,
                    "min_price": row.min_price,
                    "max_price": row.max_price,
                    "count": row.count,
                    "prev_year_avg": row.prev_year_avg,
                    "change_pct": change_pct,
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
    fn to_feature_collection_computes_change_pct() {
        let row = LandPriceAggRow {
            admin_code: "13101".into(),
            pref_name: "東京都".into(),
            city_name: "千代田区".into(),
            geometry: json!({"type": "MultiPolygon", "coordinates": []}),
            avg_price: 1100.0,
            median_price: 1000.0,
            min_price: 500.0,
            max_price: 2000.0,
            count: 10,
            prev_year_avg: 1000.0,
        };
        let fc = to_feature_collection(vec![row]);
        let features = fc["features"].as_array().unwrap();
        assert_eq!(features.len(), 1);
        let props = &features[0]["properties"];
        assert_eq!(props["change_pct"], 10.0);
        assert_eq!(props["admin_code"], "13101");
    }

    #[test]
    fn geometry_passed_through_directly() {
        let geom = json!({"type": "MultiPolygon", "coordinates": [[[[139.0, 35.0]]]]});
        let row = LandPriceAggRow {
            admin_code: "13101".into(),
            pref_name: "東京都".into(),
            city_name: "千代田区".into(),
            geometry: geom.clone(),
            avg_price: 100.0,
            median_price: 100.0,
            min_price: 100.0,
            max_price: 100.0,
            count: 1,
            prev_year_avg: 0.0,
        };
        let fc = to_feature_collection(vec![row]);
        assert_eq!(fc["features"][0]["geometry"], geom);
    }
}

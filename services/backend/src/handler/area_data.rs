use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::Value;

use crate::handler::error::AppError;
use crate::handler::request::AreaDataQuery;
use crate::handler::response::{FeatureCollectionDto, geo_feature_to_dto};
use crate::usecase::get_area_data::GetAreaDataUsecase;

/// `GET /api/area-data?south=&west=&north=&east=&layers=landprice,zoning,...`
///
/// Returns GeoJSON FeatureCollections keyed by layer name.
#[tracing::instrument(skip(usecase), fields(endpoint = "area-data"))]
pub async fn get_area_data(
    State(usecase): State<Arc<GetAreaDataUsecase>>,
    Query(params): Query<AreaDataQuery>,
) -> Result<Json<Value>, AppError> {
    let (bbox, layers) = params.into_domain()?;
    tracing::debug!(
        south = bbox.south(),
        west = bbox.west(),
        north = bbox.north(),
        east = bbox.east(),
        layers = ?layers.iter().map(|l| l.as_str()).collect::<Vec<_>>(),
        "area-data request parsed"
    );
    let result = usecase.execute(&bbox, &layers).await?;

    let total_feature_count: usize = result.values().map(Vec::len).sum();
    let layer_count = result.len();
    tracing::info!(
        feature_count = total_feature_count,
        layer_count,
        "area-data response ready"
    );

    let mut map = serde_json::Map::new();
    for (layer, features) in result {
        let feature_count = features.len();
        tracing::debug!(
            layer = layer.as_str(),
            feature_count,
            "layer features fetched"
        );
        let fc = FeatureCollectionDto::new(features.into_iter().map(geo_feature_to_dto).collect());
        map.insert(
            layer.as_str().to_string(),
            serde_json::to_value(fc).expect("FeatureCollection serialization is infallible"),
        );
    }

    Ok(Json(Value::Object(map)))
}

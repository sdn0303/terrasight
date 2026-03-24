use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};
use serde_json::Value;

use crate::domain::value_object::LayerType;
use crate::handler::error::AppError;
use crate::handler::request::AreaDataQuery;
use crate::handler::response::{LayerResponseDto, geo_feature_to_dto, point_feature_to_polygon};
use crate::usecase::get_area_data::GetAreaDataUsecase;

/// `GET /api/area-data?south=&west=&north=&east=&layers=landprice,zoning,...&zoom=14`
///
/// Returns a JSON object keyed by layer name. Each value is a `LayerResponseDto`
/// (a GeoJSON FeatureCollection enriched with `truncated`, `count`, and `limit`
/// metadata fields). Land price features are converted from Point to Polygon.
#[tracing::instrument(skip(usecase), fields(endpoint = "area-data"))]
pub async fn get_area_data(
    State(usecase): State<Arc<GetAreaDataUsecase>>,
    Query(params): Query<AreaDataQuery>,
) -> Result<Json<Value>, AppError> {
    let (bbox, layers, zoom) = params.into_domain()?;
    tracing::debug!(
        south = bbox.south(),
        west = bbox.west(),
        north = bbox.north(),
        east = bbox.east(),
        zoom,
        layers = ?layers.iter().map(|l| l.as_str()).collect::<Vec<_>>(),
        "area-data request parsed"
    );
    let result = usecase.execute(&bbox, &layers, zoom).await?;

    let total_feature_count: usize = result.values().map(|lr| lr.features.len()).sum();
    let layer_count = result.len();
    tracing::info!(
        feature_count = total_feature_count,
        layer_count,
        "area-data response ready"
    );

    let mut map = serde_json::Map::new();
    for (layer, layer_result) in result {
        let feature_count = layer_result.features.len();
        let truncated = layer_result.truncated;
        let limit = layer_result.limit;
        tracing::debug!(
            layer = layer.as_str(),
            feature_count,
            truncated,
            limit,
            "layer features fetched"
        );

        let mut feature_dtos: Vec<_> = layer_result
            .features
            .into_iter()
            .map(geo_feature_to_dto)
            .collect();

        // Convert land price points to small polygon squares for better map visibility.
        if layer == LayerType::LandPrice {
            for f in &mut feature_dtos {
                point_feature_to_polygon(f);
            }
        }

        let response_dto = LayerResponseDto::new(feature_dtos, truncated, limit);
        map.insert(
            layer.as_str().to_string(),
            serde_json::to_value(response_dto)
                .expect("LayerResponseDto serialization is infallible"),
        );
    }

    Ok(Json(Value::Object(map)))
}

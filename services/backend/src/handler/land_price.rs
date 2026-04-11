use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::LandPriceQuery;
use crate::handler::response::{LayerResponseDto, geo_feature_to_dto, point_feature_to_polygon};
use crate::usecase::get_land_prices::GetLandPricesUsecase;

/// `GET /api/v1/land-prices?year={year}&bbox={sw_lng},{sw_lat},{ne_lng},{ne_lat}&zoom={zoom}`
///
/// Returns a GeoJSON `LayerResponseDto` (FeatureCollection with truncation metadata)
/// of land price polygons within the requested bounding box for the given year.
///
/// Land price point geometries are converted to small ~30m × 30m polygon squares
/// for better visual discoverability on the MapLibre GL map.
///
/// Each feature's `properties` object contains:
/// - `id` — database row identifier
/// - `price_per_sqm` — land price per square metre (JPY)
/// - `address` — human-readable address string
/// - `land_use` — land use classification (nullable)
/// - `year` — survey year
///
/// # Errors
///
/// - `400 Bad Request` — invalid `year` or unparseable / out-of-range `bbox`
/// - `503 Service Unavailable` — database error
#[tracing::instrument(skip(usecase), fields(endpoint = "v1/land-prices"))]
pub async fn get_land_prices(
    State(usecase): State<Arc<GetLandPricesUsecase>>,
    Query(params): Query<LandPriceQuery>,
) -> Result<Json<LayerResponseDto>, AppError> {
    let (year, bbox, zoom) = params.into_domain()?;

    tracing::debug!(
        year = year.value(),
        south = bbox.south(),
        west = bbox.west(),
        north = bbox.north(),
        east = bbox.east(),
        zoom = zoom.get(),
        "land-prices request parsed"
    );

    let layer_result = usecase.execute(year, bbox, zoom).await?;

    tracing::info!(
        feature_count = layer_result.features.len(),
        truncated = layer_result.truncated,
        limit = layer_result.limit,
        "land-prices response ready"
    );

    let mut feature_dtos: Vec<_> = layer_result
        .features
        .into_iter()
        .map(geo_feature_to_dto)
        .collect();

    for f in &mut feature_dtos {
        point_feature_to_polygon(f);
    }

    Ok(Json(LayerResponseDto::new(
        feature_dtos,
        layer_result.truncated,
        layer_result.limit,
    )))
}

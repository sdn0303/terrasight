use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::LandPriceAllYearsQuery;
use crate::handler::response::{LayerResponseDto, geo_feature_to_dto, point_feature_to_polygon};
use crate::usecase::get_land_prices::GetLandPricesUsecase;

/// `GET /api/v1/land-prices/all-years?bbox={sw_lng},{sw_lat},{ne_lng},{ne_lat}&from={year}&to={year}&zoom={zoom}`
///
/// Returns a GeoJSON `LayerResponseDto` (FeatureCollection with truncation metadata)
/// of land price polygons for the full year range. Each feature's `properties.year`
/// field is populated so that clients can filter the single-response GeoJSON with
/// MapLibre `setFilter` without refetching when scrubbing the time machine slider.
///
/// Point geometries are converted to small polygon squares for 3D extrusion.
///
/// # Errors
///
/// - `400 Bad Request` — invalid year range, unparseable / out-of-range `bbox`
/// - `503 Service Unavailable` — database error
#[tracing::instrument(skip(usecase), fields(endpoint = "v1/land-prices/all-years"))]
pub async fn get_land_prices_all_years(
    State(usecase): State<Arc<GetLandPricesUsecase>>,
    Query(params): Query<LandPriceAllYearsQuery>,
) -> Result<Json<LayerResponseDto>, AppError> {
    let (from_year, to_year, bbox, zoom) = params.into_domain()?;

    tracing::debug!(
        from_year = from_year.value(),
        to_year = to_year.value(),
        south = bbox.south(),
        west = bbox.west(),
        north = bbox.north(),
        east = bbox.east(),
        zoom = zoom.get(),
        "land-prices/all-years request parsed"
    );

    let layer_result = usecase
        .execute_all_years(from_year, to_year, bbox, zoom)
        .await?;

    tracing::info!(
        feature_count = layer_result.features.len(),
        truncated = layer_result.truncated,
        limit = layer_result.limit,
        "land-prices/all-years response ready"
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

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::domain::value_object::LayerType;
use crate::handler::error::AppError;
use crate::handler::request::LandPriceAllYearsQuery;
use crate::handler::response::LayerResponseDto;
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
    let (from_year, to_year, bbox, zoom) = params.into_domain().inspect(|(f, t, b, z)| {
        tracing::debug!(
            from_year = f.value(),
            to_year = t.value(),
            south = b.south(),
            west = b.west(),
            north = b.north(),
            east = b.east(),
            zoom = z.get(),
            "land-prices/all-years request parsed"
        )
    })?;

    usecase
        .execute_all_years(from_year, to_year, bbox, zoom)
        .await
        .inspect(|lr| {
            tracing::info!(
                feature_count = lr.features.len(),
                truncated = lr.truncated,
                limit = lr.limit,
                "land-prices/all-years response ready"
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "land-prices/all-years fetch failed"))
        .map(|lr| LayerResponseDto::from_layer_result(lr, LayerType::LandPrice))
        .map(Json)
        .map_err(Into::into)
}

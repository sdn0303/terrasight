use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::LandPriceQuery;
use crate::handler::response::{FeatureCollectionDto, geo_feature_to_dto};
use crate::usecase::get_land_prices::GetLandPricesUsecase;

/// `GET /api/v1/land-prices?year={year}&bbox={sw_lng},{sw_lat},{ne_lng},{ne_lat}`
///
/// Returns a GeoJSON `FeatureCollection` of land price points within the
/// requested bounding box for the given year.
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
) -> Result<Json<FeatureCollectionDto>, AppError> {
    let (year, bbox) = params.into_domain()?;

    tracing::debug!(
        year = year.value(),
        south = bbox.south(),
        west = bbox.west(),
        north = bbox.north(),
        east = bbox.east(),
        "land-prices request parsed"
    );

    let features = usecase.execute(year, bbox).await?;

    tracing::info!(feature_count = features.len(), "land-prices response ready");

    let feature_dtos = features.into_iter().map(geo_feature_to_dto).collect();
    Ok(Json(FeatureCollectionDto::new(feature_dtos)))
}

//! `GET /api/v1/land-prices` handler.
//!
//! Returns land price point data as GeoJSON polygon features for a single
//! survey year within a bounding box. Point geometries are converted to
//! ~30 m² squares for map extrusion. Delegates to [`GetLandPricesUsecase`].

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::domain::model::LayerType;
use crate::handler::error::AppError;
use crate::handler::request::LandPriceQuery;
use crate::handler::response::LayerResponseDto;
use crate::usecase::get_land_prices::GetLandPricesUsecase;

/// Handles `GET /api/v1/land-prices`.
///
/// Query parameters: `year` (integer), `bbox` as a comma-separated string
/// `sw_lng,sw_lat,ne_lng,ne_lat` (longitude-first, RFC 7946), optional
/// `zoom` (default `14`), and optional `pref_code`.
///
/// Returns a [`LayerResponseDto`] — a GeoJSON `FeatureCollection` — where
/// each feature's `properties` object contains:
/// - `id` — database row identifier
/// - `price_per_sqm` — land price per square metre (JPY)
/// - `address` — human-readable address string
/// - `land_use` — land use classification (nullable)
/// - `year` — survey year
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
pub(crate) async fn get_land_prices(
    State(usecase): State<Arc<GetLandPricesUsecase>>,
    Query(params): Query<LandPriceQuery>,
) -> Result<Json<LayerResponseDto>, AppError> {
    let (year, bbox, zoom, pref_code) = params.into_domain().inspect(|(y, b, z, _)| {
        tracing::debug!(
            year = y.value(),
            south = b.south(),
            west = b.west(),
            north = b.north(),
            east = b.east(),
            zoom = z.get(),
            "land-prices request parsed"
        )
    })?;

    usecase
        .execute(year, bbox, zoom, pref_code.as_ref())
        .await
        .inspect(|lr| {
            tracing::info!(
                feature_count = lr.features.len(),
                truncated = lr.truncated,
                limit = lr.limit,
                "land-prices response ready"
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "land-prices fetch failed"))
        .map(|lr| LayerResponseDto::from_layer_result(lr, LayerType::LandPrice))
        .map(Json)
        .map_err(Into::into)
}

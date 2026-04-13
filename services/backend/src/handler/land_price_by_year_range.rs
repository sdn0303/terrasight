//! `GET /api/v1/land-prices/by-year-range` handler.
//!
//! Returns all land price records across a year range in a single response.
//! Each feature carries a `properties.year` field so that MapLibre GL clients
//! can animate the time-machine slider with `setFilter` — avoiding repeated
//! round trips when the user scrubs through years.
//!
//! The URL path uses `/all-years` for backwards compatibility with the
//! frontend; internally the handler and usecase are named `by_year_range`.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::domain::value_object::LayerType;
use crate::handler::error::AppError;
use crate::handler::request::LandPriceByYearRangeQuery;
use crate::handler::response::LayerResponseDto;
use crate::usecase::get_land_prices_by_year_range::GetLandPricesByYearRangeUsecase;

/// Handles `GET /api/v1/land-prices/by-year-range`.
///
/// Query parameters: `bbox` (comma-separated `sw_lng,sw_lat,ne_lng,ne_lat`),
/// optional `from` and `to` year integers (defaults to
/// `DEFAULT_FROM_YEAR..=DEFAULT_TO_YEAR`), and optional `zoom` (default `14`).
///
/// Returns a [`LayerResponseDto`] with all matching land-price polygon
/// features. Point geometries are converted to ~30 m² squares. The
/// `properties.year` field on each feature allows client-side time filtering
/// without re-fetching.
///
/// Returns a GeoJSON `LayerResponseDto` (FeatureCollection with truncation metadata)
/// of land price polygons for the full year range. Each feature's `properties.year`
/// field is populated so that clients can filter the single-response GeoJSON with
/// MapLibre `setFilter` without refetching when scrubbing the time machine slider.
///
/// The URL path preserves `/all-years` for frontend backwards compatibility even
/// though the handler and usecase are now named `by_year_range`.
///
/// Point geometries are converted to small polygon squares for 3D extrusion.
///
/// # Errors
///
/// - `400 Bad Request` — invalid year range, unparseable / out-of-range `bbox`
/// - `503 Service Unavailable` — database error
#[tracing::instrument(skip(usecase), fields(endpoint = "v1/land-prices/all-years"))]
pub(crate) async fn get_land_prices_by_year_range(
    State(usecase): State<Arc<GetLandPricesByYearRangeUsecase>>,
    Query(params): Query<LandPriceByYearRangeQuery>,
) -> Result<Json<LayerResponseDto>, AppError> {
    let (from_year, to_year, bbox, zoom, pref_code) =
        params.into_domain().inspect(|(f, t, b, z, _)| {
            tracing::debug!(
                from_year = f.value(),
                to_year = t.value(),
                south = b.south(),
                west = b.west(),
                north = b.north(),
                east = b.east(),
                zoom = z.get(),
                "land-prices/by-year-range request parsed"
            )
        })?;

    usecase
        .execute(from_year, to_year, bbox, zoom, pref_code.as_ref())
        .await
        .inspect(|lr| {
            tracing::info!(
                feature_count = lr.features.len(),
                truncated = lr.truncated,
                limit = lr.limit,
                "land-prices/by-year-range response ready"
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "land-prices/by-year-range fetch failed"))
        .map(|lr| LayerResponseDto::from_layer_result(lr, LayerType::LandPrice))
        .map(Json)
        .map_err(Into::into)
}

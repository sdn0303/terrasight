use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};

use crate::handler::error::AppError;
use crate::handler::request::AreaDataQuery;
use crate::handler::response::AreaDataResponseDto;
use crate::usecase::get_area_data::GetAreaDataUsecase;

/// `GET /api/area-data?south=&west=&north=&east=&layers=landprice,zoning,...&zoom=14`
///
/// Returns a JSON object keyed by layer name. Each value is a `LayerResponseDto`
/// (a GeoJSON FeatureCollection enriched with `truncated`, `count`, and `limit`
/// metadata fields). Land price features are converted from Point to Polygon.
#[tracing::instrument(skip(usecase), fields(endpoint = "area-data"))]
pub(crate) async fn get_area_data(
    State(usecase): State<Arc<GetAreaDataUsecase>>,
    Query(params): Query<AreaDataQuery>,
) -> Result<Json<AreaDataResponseDto>, AppError> {
    let (bbox, layers, zoom, pref_code) = params.into_domain().inspect(|(b, l, z, _)| {
        tracing::debug!(
            south = b.south(),
            west = b.west(),
            north = b.north(),
            east = b.east(),
            zoom = z.get(),
            layer_count = l.len(),
            "area-data request parsed"
        )
    })?;

    usecase
        .execute(&bbox, &layers, zoom, pref_code.as_ref())
        .await
        .inspect(|result| {
            tracing::info!(
                layer_count = result.len(),
                feature_count = result.values().map(|lr| lr.features.len()).sum::<usize>(),
                "area-data response ready"
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "area-data fetch failed"))
        .map(AreaDataResponseDto::from_domain)
        .map(Json)
        .map_err(Into::into)
}

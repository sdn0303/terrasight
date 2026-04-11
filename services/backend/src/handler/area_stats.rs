use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
};
use serde::Serialize;

use crate::handler::error::AppError;
use crate::handler::request::AreaStatsQuery;
use crate::usecase::get_area_stats::GetAreaStatsUsecase;

/// Response for `GET /api/area-stats`.
#[derive(Debug, Serialize)]
pub struct AreaStatsResponse {
    pub code: String,
    pub name: String,
    pub level: String,
    pub land_price: AreaLandPriceDto,
    pub risk: AreaRiskDto,
    pub facilities: AreaFacilitiesDto,
}

#[derive(Debug, Serialize)]
pub struct AreaLandPriceDto {
    pub avg_per_sqm: Option<f64>,
    pub median_per_sqm: Option<f64>,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct AreaRiskDto {
    pub flood_area_ratio: f64,
    pub composite_risk: f64,
}

#[derive(Debug, Serialize)]
pub struct AreaFacilitiesDto {
    pub schools: i64,
    pub medical: i64,
}

/// `GET /api/area-stats?code={admin_code}`
///
/// Returns aggregated statistics for the given administrative area.
/// `code` is a 2-digit prefecture code (e.g. `"13"`) or 5-digit municipality
/// code (e.g. `"13105"` for Bunkyo-ku, Tokyo).
#[tracing::instrument(skip(usecase), fields(endpoint = "area-stats"))]
pub async fn get_area_stats(
    State(usecase): State<Arc<GetAreaStatsUsecase>>,
    Query(params): Query<AreaStatsQuery>,
) -> Result<Json<AreaStatsResponse>, AppError> {
    let code = params.into_domain()?;
    let stats = usecase.execute(&code).await?;
    tracing::info!(
        code = %stats.code,
        level = %stats.level,
        land_price_count = stats.land_price.count,
        "area-stats response ready"
    );
    Ok(Json(AreaStatsResponse {
        code: stats.code,
        name: stats.name,
        level: stats.level,
        land_price: AreaLandPriceDto {
            avg_per_sqm: stats.land_price.avg_per_sqm,
            median_per_sqm: stats.land_price.median_per_sqm,
            count: stats.land_price.count,
        },
        risk: AreaRiskDto {
            flood_area_ratio: stats.risk.flood_area_ratio,
            composite_risk: stats.risk.composite_risk,
        },
        facilities: AreaFacilitiesDto {
            schools: stats.facilities.schools,
            medical: stats.facilities.medical,
        },
    }))
}

//! Shared sqlx `FromRow` structs used across multiple repositories.

use serde_json::json;

use crate::domain::entity::{GeoFeature, LandPriceStats};
use crate::infra::geo_convert::to_geo_feature;

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct CountRow {
    pub(crate) count: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct AreaRow {
    pub(crate) area: f64,
}

#[derive(Debug, sqlx::FromRow)]
pub(crate) struct LandPriceStatsRow {
    pub(crate) avg_price: Option<f64>,
    pub(crate) median_price: Option<f64>,
    pub(crate) min_price: Option<i64>,
    pub(crate) max_price: Option<i64>,
    pub(crate) count: i64,
}

impl From<LandPriceStatsRow> for LandPriceStats {
    fn from(row: LandPriceStatsRow) -> Self {
        Self {
            avg_per_sqm: row.avg_price,
            median_per_sqm: row.median_price,
            min_per_sqm: row.min_price,
            max_per_sqm: row.max_price,
            count: row.count,
        }
    }
}

/// Raw row returned by land-price spatial queries.
///
/// Shared between [`pg_area_repository`] and [`pg_land_price_repository`] to
/// avoid duplicating an identical struct.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct LandPriceFeatureRow {
    pub(crate) id: i64,
    pub(crate) price_per_sqm: i32,
    pub(crate) address: String,
    pub(crate) land_use: Option<String>,
    pub(crate) survey_year: i16,
    pub(crate) geometry: serde_json::Value,
}

impl From<LandPriceFeatureRow> for GeoFeature {
    fn from(row: LandPriceFeatureRow) -> Self {
        to_geo_feature(
            row.geometry,
            json!({
                "id": row.id,
                "price_per_sqm": row.price_per_sqm,
                "address": row.address,
                "land_use": row.land_use,
                "year": row.survey_year,
            }),
        )
    }
}

//! Shared sqlx [`FromRow`](sqlx::FromRow) structs used across multiple repositories.
//!
//! Centralising these structs eliminates duplicate definitions and ensures
//! that the same column aliases (`count`, `area`, …) are projected consistently
//! across all SQL queries that produce them.

use serde_json::json;

use crate::domain::model::{GeoFeature, LandPriceStats};
use crate::infra::geo_convert::to_geo_feature;

/// A single-column row produced by `SELECT COUNT(*) AS count …` queries.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct CountRow {
    /// The row count returned by the query.
    pub(crate) count: i64,
}

/// A single-column row produced by `SELECT ST_Area(…) AS area …` queries.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct AreaRow {
    /// The computed area in square metres (when cast to `geography`) or
    /// square degrees (when left as `geometry`).
    pub(crate) area: f64,
}

/// Aggregate land-price statistics returned by `AVG / PERCENTILE_CONT / MIN / MAX / COUNT` queries.
///
/// All price fields are `Option` because the aggregate functions return `NULL`
/// when the filtered result set is empty.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct LandPriceStatsRow {
    /// Mean price per square metre across the filtered rows.
    pub(crate) avg_price: Option<f64>,
    /// Median (50th percentile) price per square metre.
    pub(crate) median_price: Option<f64>,
    /// Minimum price per square metre.
    pub(crate) min_price: Option<i64>,
    /// Maximum price per square metre.
    pub(crate) max_price: Option<i64>,
    /// Total number of land-price records in the result set.
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
/// Shared between [`pg_area_repository`](crate::infra::pg_area_repository) and
/// [`pg_land_price_repository`](crate::infra::pg_land_price_repository) to
/// avoid duplicating an identical struct. The `geometry` column must be
/// projected as `ST_AsGeoJSON(geom)::jsonb AS geometry`.
#[derive(Debug, sqlx::FromRow)]
pub(crate) struct LandPriceFeatureRow {
    /// Primary key of the `land_prices` row.
    pub(crate) id: i64,
    /// Land price in yen per square metre.
    pub(crate) price_per_sqm: i32,
    /// Postal address string for the survey point.
    pub(crate) address: String,
    /// MLIT land-use category code (may be absent for older records).
    pub(crate) land_use: Option<String>,
    /// Year the survey was conducted.
    pub(crate) survey_year: i16,
    /// GeoJSON geometry produced by `ST_AsGeoJSON(geom)::jsonb`.
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

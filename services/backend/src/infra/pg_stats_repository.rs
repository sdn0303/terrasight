//! PostgreSQL + PostGIS implementation of [`StatsRepository`].
//!
//! Implements [`StatsRepository`](crate::domain::repository::StatsRepository)
//! which aggregates four distinct statistics for a bounding box:
//!
//! - **Land price stats** — `AVG`, `PERCENTILE_CONT(0.5)`, `MIN`, `MAX`, `COUNT`
//!   on `land_prices` for the latest `survey_year` in the bbox.
//! - **Risk stats** — `ST_Area(ST_Intersection(geom, envelope))` for the `flood_risk`
//!   and `steep_slope` tables, normalised by the bbox area
//!   (`ST_Area(ST_MakeEnvelope(...)::geography)`). The composite risk score is
//!   a weighted sum using [`STATS_RISK_WEIGHT_FLOOD`](crate::domain::constants::STATS_RISK_WEIGHT_FLOOD)
//!   and [`STATS_RISK_WEIGHT_STEEP`](crate::domain::constants::STATS_RISK_WEIGHT_STEEP).
//! - **Facility counts** — `COUNT(*)` on `schools` and `medical_facilities`.
//! - **Zoning distribution** — area-weighted `ratio` per `zone_type` using a
//!   window-function `SUM OVER ()` pattern (single-pass, no subquery needed).
//!
//! All queries enforce [`STATS_QUERY_TIMEOUT`] via
//! [`run_query`](crate::infra::query_helpers::run_query).

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use sqlx::PgPool;
use terrasight_geo::GeoBBox;
use terrasight_server::db::spatial::bind_bbox;

use crate::domain::constants::{STATS_RISK_WEIGHT_FLOOD, STATS_RISK_WEIGHT_STEEP};
use crate::domain::entity::{FacilityStats, LandPriceStats, RiskStats};
use crate::domain::error::DomainError;
use crate::domain::repository::StatsRepository;
use crate::domain::value_object::{BBox, PrefCode};
use crate::infra::query_helpers::run_query;
use crate::infra::row_types::{AreaRow, CountRow, LandPriceStatsRow};

/// Maximum time to wait for a single stats aggregation query.
const STATS_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, sqlx::FromRow)]
struct ZoneDistRow {
    zone_type: String,
    ratio: f64,
}

/// PostgreSQL + PostGIS implementation of [`StatsRepository`](crate::domain::repository::StatsRepository).
pub(crate) struct PgStatsRepository {
    pool: PgPool,
}

impl PgStatsRepository {
    /// Create a new repository backed by the given connection pool.
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StatsRepository for PgStatsRepository {
    /// Compute land price aggregates (`avg`, `median`, `min`, `max`, `count`)
    /// for the latest `survey_year` that falls within the bounding box.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] if the query exceeds
    /// [`STATS_QUERY_TIMEOUT`], or [`DomainError::Database`] on a
    /// PostgreSQL error.
    #[tracing::instrument(skip(self))]
    async fn calc_land_price_stats(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<LandPriceStats, DomainError> {
        let geo_bbox = GeoBBox::new(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let row = run_query(
            STATS_QUERY_TIMEOUT,
            "land_price_stats query",
            bind_bbox(
                sqlx::query_as::<_, LandPriceStatsRow>(
                    r#"
            SELECT
                AVG(price_per_sqm)::float8 AS avg_price,
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY price_per_sqm)::float8 AS median_price,
                MIN(price_per_sqm)::int8 AS min_price,
                MAX(price_per_sqm)::int8 AS max_price,
                COUNT(*) AS count
            FROM land_prices
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
              AND survey_year = (SELECT MAX(survey_year) FROM land_prices WHERE ($5::text IS NULL OR pref_code = $5))
              AND ($5::text IS NULL OR pref_code = $5)
            "#,
                ),
                &geo_bbox,
            )
            .bind(pref_code.map(PrefCode::as_str))
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|row| tracing::debug!(count = row.count, "land_price_stats fetched"))?;

        Ok(row.into())
    }

    /// Compute flood-risk and steep-slope area ratios for the bounding box.
    ///
    /// Uses `ST_Area(ST_Intersection(...))` to measure the overlap area
    /// between each hazard layer and the bbox, then divides by the total
    /// bbox area. Returns early with zero ratios if the bbox has zero area.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn calc_risk_stats(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<RiskStats, DomainError> {
        let geo_bbox = GeoBBox::new(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let bbox_area_row = run_query(
            STATS_QUERY_TIMEOUT,
            "bbox_area query",
            bind_bbox(
                sqlx::query_as::<_, AreaRow>(
                    "SELECT ST_Area(ST_MakeEnvelope($1, $2, $3, $4, 4326)::geography) AS area",
                ),
                &geo_bbox,
            )
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(area = r.area, "bbox_area fetched"))?;

        let bbox_area = bbox_area_row.area;
        if bbox_area == 0.0 {
            return Ok(RiskStats {
                flood_area_ratio: 0.0,
                steep_slope_area_ratio: 0.0,
                composite_risk: 0.0,
            });
        }

        let flood_row = run_query(
            STATS_QUERY_TIMEOUT,
            "flood_area_sum query",
            bind_bbox(
                sqlx::query_as::<_, AreaRow>(
                    r#"
            SELECT COALESCE(SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography)), 0) AS area
            FROM flood_risk
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
              AND ($5::text IS NULL OR pref_code = $5)
            "#,
                ),
                &geo_bbox,
            )
            .bind(pref_code.map(PrefCode::as_str))
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(area = r.area, "flood_area_sum fetched"))?;

        let slope_row = run_query(
            STATS_QUERY_TIMEOUT,
            "steep_slope_area_sum query",
            bind_bbox(
                sqlx::query_as::<_, AreaRow>(
                    r#"
            SELECT COALESCE(SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography)), 0) AS area
            FROM steep_slope
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
              AND ($5::text IS NULL OR pref_code = $5)
            "#,
                ),
                &geo_bbox,
            )
            .bind(pref_code.map(PrefCode::as_str))
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(area = r.area, "steep_slope_area_sum fetched"))?;

        let flood_ratio = flood_row.area / bbox_area;
        let slope_ratio = slope_row.area / bbox_area;
        let composite =
            flood_ratio * STATS_RISK_WEIGHT_FLOOD + slope_ratio * STATS_RISK_WEIGHT_STEEP;
        tracing::debug!(flood_ratio = %format!("{:.4}", flood_ratio), slope_ratio = %format!("{:.4}", slope_ratio), "risk_stats computed");

        Ok(RiskStats {
            flood_area_ratio: flood_ratio,
            steep_slope_area_ratio: slope_ratio,
            composite_risk: composite,
        })
    }

    /// Count schools and medical facilities within the bounding box.
    ///
    /// Runs two `COUNT(*)` queries in sequence against the `schools` and
    /// `medical_facilities` tables. Both are bounded by [`STATS_QUERY_TIMEOUT`].
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn count_facilities(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<FacilityStats, DomainError> {
        let geo_bbox = GeoBBox::new(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let schools = run_query(
            STATS_QUERY_TIMEOUT,
            "schools_count query",
            bind_bbox(
                sqlx::query_as::<_, CountRow>(
                    "SELECT COUNT(*) AS count FROM schools WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326)) AND ($5::text IS NULL OR pref_code = $5)",
                ),
                &geo_bbox,
            )
            .bind(pref_code.map(PrefCode::as_str))
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(count = r.count, "schools_count fetched"))?;

        let medical = run_query(
            STATS_QUERY_TIMEOUT,
            "medical_count query",
            bind_bbox(
                sqlx::query_as::<_, CountRow>(
                    "SELECT COUNT(*) AS count FROM medical_facilities WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326)) AND ($5::text IS NULL OR pref_code = $5)",
                ),
                &geo_bbox,
            )
            .bind(pref_code.map(PrefCode::as_str))
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(count = r.count, "medical_count fetched"))?;

        tracing::debug!(
            schools = schools.count,
            medical = medical.count,
            "facility_counts fetched"
        );

        Ok(FacilityStats {
            schools: schools.count,
            medical: medical.count,
        })
    }

    /// Compute the area-weighted zoning distribution within the bounding box.
    ///
    /// Uses a window-function ratio:
    /// `SUM(area) / SUM(SUM(area)) OVER ()` per `zone_type` so that the entire
    /// distribution is computed in a single pass. Returns a `HashMap<zone_type, ratio>`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn calc_zoning_distribution(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<HashMap<String, f64>, DomainError> {
        let geo_bbox = GeoBBox::new(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let rows = run_query(
            STATS_QUERY_TIMEOUT,
            "zoning_distribution query",
            bind_bbox(
                sqlx::query_as::<_, ZoneDistRow>(
                    r#"
            SELECT zone_type,
                   SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography))
                   / NULLIF(SUM(SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography))) OVER (), 0)
                   AS ratio
            FROM zoning
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
              AND ($5::text IS NULL OR pref_code = $5)
            GROUP BY zone_type
            ORDER BY ratio DESC
            "#,
                ),
                &geo_bbox,
            )
            .bind(pref_code.map(PrefCode::as_str))
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| tracing::debug!(zone_types = rows.len(), "zoning_distribution fetched"))?;

        Ok(rows.into_iter().map(|r| (r.zone_type, r.ratio)).collect())
    }
}

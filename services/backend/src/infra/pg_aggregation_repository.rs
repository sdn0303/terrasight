//! PostgreSQL repository for polygon aggregation queries.
//!
//! Implements [`AggregationRepository`](crate::domain::repository::AggregationRepository)
//! using PostGIS spatial joins between `admin_boundaries` and domain tables.
//! All queries are wrapped with [`run_query`] for timeout enforcement.
//!
//! ## Query design
//!
//! **Land price aggregation**: Pre-filters `land_prices` to the bbox via
//! `ST_Intersects` in a `MATERIALIZED` CTE (`lp_bbox`), then joins with
//! `ST_Contains` for point-in-polygon. Uses `FILTER (WHERE …)` to split
//! current-year vs previous-year aggregates in a single pass — no LATERAL.
//!
//! **Transaction aggregation**: Joins `admin_boundaries` with
//! `mv_transaction_summary` (materialized view) via `city_code` + `pref_code`
//! for the latest year. The `pref_code` condition enables partition pruning
//! on the underlying `transaction_prices` partitioned table.

use std::time::Duration;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::error::DomainError;
use crate::domain::model::{BBox, LandPriceAggRow, PrefCode, TransactionAggRow};
use crate::domain::repository::AggregationRepository;
use crate::infra::query_helpers::run_query;

/// Maximum time to wait for an aggregation query.
const AGGREGATION_QUERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Raw sqlx row for land price aggregation.
#[derive(Debug, sqlx::FromRow)]
struct LandPriceAggSqlRow {
    admin_code: String,
    pref_name: String,
    city_name: String,
    geometry: serde_json::Value,
    avg_price: f64,
    median_price: f64,
    min_price: f64,
    max_price: f64,
    count: i32,
    prev_year_avg: f64,
}

impl From<LandPriceAggSqlRow> for LandPriceAggRow {
    fn from(r: LandPriceAggSqlRow) -> Self {
        Self {
            admin_code: r.admin_code,
            pref_name: r.pref_name,
            city_name: r.city_name,
            geometry: r.geometry,
            avg_price: r.avg_price,
            median_price: r.median_price,
            min_price: r.min_price,
            max_price: r.max_price,
            count: r.count,
            prev_year_avg: r.prev_year_avg,
        }
    }
}

/// Raw sqlx row for transaction aggregation.
#[derive(Debug, sqlx::FromRow)]
struct TransactionAggSqlRow {
    admin_code: String,
    city_name: String,
    geometry: serde_json::Value,
    tx_count: i32,
    avg_price_sqm: f64,
    avg_total_price: f64,
}

impl From<TransactionAggSqlRow> for TransactionAggRow {
    fn from(r: TransactionAggSqlRow) -> Self {
        Self {
            admin_code: r.admin_code,
            city_name: r.city_name,
            geometry: r.geometry,
            tx_count: r.tx_count,
            avg_price_sqm: r.avg_price_sqm,
            avg_total_price: r.avg_total_price,
        }
    }
}

/// PostgreSQL implementation of [`AggregationRepository`].
pub(crate) struct PgAggregationRepository {
    pool: PgPool,
}

impl PgAggregationRepository {
    /// Create a new repository backed by the given connection pool.
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AggregationRepository for PgAggregationRepository {
    /// Land price aggregation: bbox pre-filter CTE + single `LEFT JOIN` with
    /// `FILTER` to split current/previous year aggregates.
    ///
    /// Query structure (2-table JOIN):
    /// 1. `latest` CTE — `MAX(survey_year)` from `land_prices`
    /// 2. `lp_bbox` CTE — pre-filters `land_prices` to bbox + year range
    /// 3. Main query — `admin_boundaries LEFT JOIN lp_bbox` via `ST_Contains`
    #[tracing::instrument(skip(self), fields(repo = "pg_aggregation"))]
    async fn land_price_aggregation(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<Vec<LandPriceAggRow>, DomainError> {
        let pref_str = pref_code.map(|p| p.as_str().to_owned());

        let rows: Vec<LandPriceAggSqlRow> = run_query(
            AGGREGATION_QUERY_TIMEOUT,
            "land_price_aggregation",
            sqlx::query_as(
                r#"
                WITH latest AS (
                    SELECT MAX(survey_year) AS yr
                    FROM land_prices
                    WHERE ($5::text IS NULL OR pref_code = $5)
                ),
                lp_bbox AS MATERIALIZED (
                    SELECT lp.id, lp.price_per_sqm, lp.survey_year, lp.geom
                    FROM land_prices lp
                    CROSS JOIN latest ly
                    WHERE ST_Intersects(lp.geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                      AND ($5::text IS NULL OR lp.pref_code = $5)
                      AND lp.survey_year IN (ly.yr, ly.yr - 1)
                )
                SELECT
                    ab.admin_code,
                    ab.pref_name,
                    ab.city_name,
                    ST_AsGeoJSON(ab.geom)::jsonb                                              AS geometry,
                    COALESCE(AVG(lp.price_per_sqm)
                        FILTER (WHERE lp.survey_year = ly.yr), 0)::float8                     AS avg_price,
                    COALESCE(percentile_cont(0.5) WITHIN GROUP (ORDER BY lp.price_per_sqm)
                        FILTER (WHERE lp.survey_year = ly.yr), 0)::float8                     AS median_price,
                    COALESCE(MIN(lp.price_per_sqm)
                        FILTER (WHERE lp.survey_year = ly.yr), 0)::float8                     AS min_price,
                    COALESCE(MAX(lp.price_per_sqm)
                        FILTER (WHERE lp.survey_year = ly.yr), 0)::float8                     AS max_price,
                    (COUNT(lp.id) FILTER (WHERE lp.survey_year = ly.yr))::int4                AS count,
                    COALESCE(AVG(lp.price_per_sqm)
                        FILTER (WHERE lp.survey_year = ly.yr - 1), 0)::float8                 AS prev_year_avg
                FROM admin_boundaries ab
                CROSS JOIN latest ly
                LEFT JOIN lp_bbox lp
                    ON ST_Contains(ab.geom, lp.geom)
                WHERE ab.level = 'municipality'
                    AND ST_Intersects(ab.geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                    AND ($5::text IS NULL OR ab.pref_code = $5)
                GROUP BY ab.admin_code, ab.pref_name, ab.city_name, ab.geom
                HAVING (COUNT(lp.id) FILTER (WHERE lp.survey_year = ly.yr)) > 0
                "#,
            )
            .bind(bbox.west())
            .bind(bbox.south())
            .bind(bbox.east())
            .bind(bbox.north())
            .bind(pref_str.as_deref())
            .fetch_all(&self.pool),
        )
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "land_price_aggregation query failed"))?;

        Ok(rows.into_iter().map(LandPriceAggRow::from).collect())
    }

    /// Transaction aggregation: joins `admin_boundaries` with
    /// `mv_transaction_summary` for the latest year.
    ///
    /// Uses `pref_code` in the join condition for partition pruning on
    /// the underlying `transaction_prices` partitioned table.
    #[tracing::instrument(skip(self), fields(repo = "pg_aggregation"))]
    async fn transaction_aggregation(
        &self,
        bbox: &BBox,
        pref_code: Option<&PrefCode>,
    ) -> Result<Vec<TransactionAggRow>, DomainError> {
        let pref_str = pref_code.map(|p| p.as_str().to_owned());

        let rows: Vec<TransactionAggSqlRow> = run_query(
            AGGREGATION_QUERY_TIMEOUT,
            "transaction_aggregation",
            sqlx::query_as(
                r#"
                WITH latest_tx AS (
                    SELECT MAX(transaction_year) AS yr
                    FROM mv_transaction_summary
                    WHERE ($5::text IS NULL OR pref_code = $5)
                )
                SELECT
                    ab.admin_code,
                    ab.city_name,
                    ST_AsGeoJSON(ab.geom)::jsonb                              AS geometry,
                    COALESCE(SUM(mvt.tx_count), 0)::int4                      AS tx_count,
                    COALESCE(
                        SUM(mvt.avg_price_sqm::bigint * mvt.tx_count) /
                            NULLIF(SUM(mvt.tx_count), 0),
                        0
                    )::float8                                                  AS avg_price_sqm,
                    COALESCE(
                        SUM(mvt.avg_total_price * mvt.tx_count) /
                            NULLIF(SUM(mvt.tx_count), 0),
                        0
                    )::float8                                                  AS avg_total_price
                FROM admin_boundaries ab
                CROSS JOIN latest_tx ly
                LEFT JOIN mv_transaction_summary mvt
                    ON mvt.city_code = ab.admin_code
                    AND mvt.pref_code = ab.pref_code
                    AND mvt.transaction_year = ly.yr
                WHERE ab.level = 'municipality'
                    AND ST_Intersects(ab.geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                    AND ($5::text IS NULL OR ab.pref_code = $5)
                GROUP BY ab.admin_code, ab.city_name, ab.geom
                HAVING COALESCE(SUM(mvt.tx_count), 0) > 0
                "#,
            )
            .bind(bbox.west())
            .bind(bbox.south())
            .bind(bbox.east())
            .bind(bbox.north())
            .bind(pref_str.as_deref())
            .fetch_all(&self.pool),
        )
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "transaction_aggregation query failed"))?;

        Ok(rows.into_iter().map(TransactionAggRow::from).collect())
    }
}

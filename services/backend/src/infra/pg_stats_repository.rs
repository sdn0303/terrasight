use std::collections::HashMap;

use async_trait::async_trait;
use realestate_db::spatial::bind_bbox;
use sqlx::{FromRow, PgPool};

use super::map_db_err;
use crate::domain::constants::{STATS_RISK_WEIGHT_FLOOD, STATS_RISK_WEIGHT_STEEP};
use crate::domain::entity::{FacilityStats, LandPriceStats, RiskStats};
use crate::domain::error::DomainError;
use crate::domain::repository::StatsRepository;
use crate::domain::value_object::BBox;

#[derive(Debug, FromRow)]
struct LandPriceStatsRow {
    avg_price: Option<f64>,
    median_price: Option<f64>,
    min_price: Option<i64>,
    max_price: Option<i64>,
    count: i64,
}

impl From<LandPriceStatsRow> for LandPriceStats {
    fn from(row: LandPriceStatsRow) -> Self {
        LandPriceStats {
            avg_per_sqm: row.avg_price,
            median_per_sqm: row.median_price,
            min_per_sqm: row.min_price,
            max_per_sqm: row.max_price,
            count: row.count,
        }
    }
}

pub struct PgStatsRepository {
    pool: PgPool,
}

impl PgStatsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl StatsRepository for PgStatsRepository {
    #[tracing::instrument(skip(self))]
    async fn calc_land_price_stats(&self, bbox: &BBox) -> Result<LandPriceStats, DomainError> {
        let query = sqlx::query_as::<_, LandPriceStatsRow>(
            r#"
            SELECT
                AVG(price_per_sqm)::float8 AS avg_price,
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY price_per_sqm)::float8 AS median_price,
                MIN(price_per_sqm)::int8 AS min_price,
                MAX(price_per_sqm)::int8 AS max_price,
                COUNT(*) AS count
            FROM land_prices
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
              AND year = (SELECT MAX(year) FROM land_prices)
            "#,
        );
        let row = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(count = row.count, "land_price_stats fetched");

        Ok(row.into())
    }

    #[tracing::instrument(skip(self))]
    async fn calc_risk_stats(&self, bbox: &BBox) -> Result<RiskStats, DomainError> {
        let bbox_area_query = sqlx::query_as::<_, (f64,)>(
            "SELECT ST_Area(ST_MakeEnvelope($1, $2, $3, $4, 4326)::geography)",
        );
        let bbox_area_row = bind_bbox(
            bbox_area_query,
            bbox.west(),
            bbox.south(),
            bbox.east(),
            bbox.north(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err)?;

        let bbox_area = bbox_area_row.0;
        if bbox_area == 0.0 {
            return Ok(RiskStats {
                flood_area_ratio: 0.0,
                steep_slope_area_ratio: 0.0,
                composite_risk: 0.0,
            });
        }

        let flood_query = sqlx::query_as::<_, (f64,)>(
            r#"
            SELECT COALESCE(SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography)), 0)
            FROM flood_risk
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            "#,
        );
        let flood_row = bind_bbox(
            flood_query,
            bbox.west(),
            bbox.south(),
            bbox.east(),
            bbox.north(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err)?;

        let slope_query = sqlx::query_as::<_, (f64,)>(
            r#"
            SELECT COALESCE(SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography)), 0)
            FROM steep_slope
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            "#,
        );
        let slope_row = bind_bbox(
            slope_query,
            bbox.west(),
            bbox.south(),
            bbox.east(),
            bbox.north(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err)?;

        let flood_ratio = flood_row.0 / bbox_area;
        let slope_ratio = slope_row.0 / bbox_area;
        let composite =
            flood_ratio * STATS_RISK_WEIGHT_FLOOD + slope_ratio * STATS_RISK_WEIGHT_STEEP;
        tracing::debug!(flood_ratio = %format!("{:.4}", flood_ratio), slope_ratio = %format!("{:.4}", slope_ratio), "risk_stats computed");

        Ok(RiskStats {
            flood_area_ratio: flood_ratio,
            steep_slope_area_ratio: slope_ratio,
            composite_risk: composite,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn count_facilities(&self, bbox: &BBox) -> Result<FacilityStats, DomainError> {
        let schools_query = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM schools WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))",
        );
        let schools = bind_bbox(
            schools_query,
            bbox.west(),
            bbox.south(),
            bbox.east(),
            bbox.north(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err)?;

        let medical_query = sqlx::query_as::<_, (i64,)>(
            "SELECT COUNT(*) FROM medical_facilities WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))",
        );
        let medical = bind_bbox(
            medical_query,
            bbox.west(),
            bbox.south(),
            bbox.east(),
            bbox.north(),
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_err)?;
        tracing::debug!(
            schools = schools.0,
            medical = medical.0,
            "facility_counts fetched"
        );

        Ok(FacilityStats {
            schools: schools.0,
            medical: medical.0,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn calc_zoning_distribution(
        &self,
        bbox: &BBox,
    ) -> Result<HashMap<String, f64>, DomainError> {
        let query = sqlx::query_as::<_, (String, f64)>(
            r#"
            SELECT zone_type,
                   SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography))
                   / NULLIF(SUM(SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography))) OVER (), 0)
                   AS ratio
            FROM zoning
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            GROUP BY zone_type
            ORDER BY ratio DESC
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(zone_types = rows.len(), "zoning_distribution fetched");

        Ok(rows.into_iter().collect())
    }
}

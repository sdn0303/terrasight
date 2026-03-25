use async_trait::async_trait;
use realestate_db::spatial::bind_coord;
use sqlx::PgPool;

use super::map_db_err;
use crate::domain::entity::{MedicalStats, PriceRecord, SchoolStats, ZScoreResult};
use crate::domain::error::DomainError;
use crate::domain::repository::TlsRepository;
use crate::domain::value_object::Coord;

pub struct PgTlsRepository {
    pool: PgPool,
}

impl PgTlsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TlsRepository for PgTlsRepository {
    #[tracing::instrument(skip(self))]
    async fn find_nearest_prices(&self, coord: &Coord) -> Result<Vec<PriceRecord>, DomainError> {
        // Search radius: 1000m, SRID: 4326
        let query = sqlx::query_as::<_, (i32, i32, String, f64)>(
            r#"
            WITH nearest AS (
                SELECT address,
                       ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS dist
                FROM land_prices
                WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)
                ORDER BY dist
                LIMIT 1
            )
            SELECT lp.year, lp.price_per_sqm, lp.address,
                   ST_Distance(lp.geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS distance_m
            FROM land_prices lp
            INNER JOIN nearest n ON lp.address = n.address
            ORDER BY lp.year
            "#,
        );
        let rows = bind_coord(query, coord.lng(), coord.lat())
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), "tls nearest_prices fetched");

        Ok(rows
            .into_iter()
            .map(|(year, price, address, dist)| PriceRecord {
                year,
                price_per_sqm: price as i64,
                address,
                distance_m: dist,
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn find_flood_depth_rank(&self, coord: &Coord) -> Result<Option<i32>, DomainError> {
        // MAX depth_rank within 500m buffer. Returns NULL when no flood zone intersects.
        let query = sqlx::query_as::<_, (Option<i32>,)>(
            r#"
            SELECT MAX(depth_rank)
            FROM flood_risk
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;

        Ok(row.0)
    }

    #[tracing::instrument(skip(self))]
    async fn has_steep_slope_nearby(&self, coord: &Coord) -> Result<bool, DomainError> {
        // 500m buffer, SRID: 4326
        let query = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM steep_slope
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;

        Ok(row.0 > 0)
    }

    #[tracing::instrument(skip(self))]
    async fn find_schools_nearby(&self, coord: &Coord) -> Result<SchoolStats, DomainError> {
        // 800m radius, SRID: 4326
        let query = sqlx::query_as::<_, (i64, bool, bool)>(
            r#"
            SELECT COUNT(*),
                   COALESCE(bool_or(school_type = '小学校'), false),
                   COALESCE(bool_or(school_type = '中学校'), false)
            FROM schools
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 800)
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(
            count = row.0,
            has_primary = row.1,
            has_junior_high = row.2,
            "schools_nearby fetched"
        );

        Ok(SchoolStats {
            count_800m: row.0,
            has_primary: row.1,
            has_junior_high: row.2,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn find_medical_nearby(&self, coord: &Coord) -> Result<MedicalStats, DomainError> {
        // 1000m radius, SRID: 4326
        let query = sqlx::query_as::<_, (i64, i64, i64)>(
            r#"
            SELECT COUNT(*) FILTER (WHERE facility_type = '病院'),
                   COUNT(*) FILTER (WHERE facility_type != '病院'),
                   COALESCE(SUM(bed_count), 0)
            FROM medical_facilities
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(
            hospitals = row.0,
            clinics = row.1,
            beds = row.2,
            "medical_nearby fetched"
        );

        Ok(MedicalStats {
            hospital_count: row.0,
            clinic_count: row.1,
            total_beds: row.2,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn find_zoning_far(&self, coord: &Coord) -> Result<Option<f64>, DomainError> {
        // Find the zoning polygon that contains the point; return its floor_area_ratio.
        let query = sqlx::query_as::<_, (Option<f64>,)>(
            r#"
            SELECT floor_area_ratio::double precision
            FROM zoning
            WHERE ST_Contains(geom, ST_SetSRID(ST_MakePoint($1, $2), 4326))
            LIMIT 1
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err)?;

        Ok(row.and_then(|(far,)| far))
    }

    #[tracing::instrument(skip(self))]
    async fn calc_price_z_score(&self, coord: &Coord) -> Result<ZScoreResult, DomainError> {
        // Step 1: Find the zone_type containing the point.
        // Step 2: For all land_prices in that zone_type (latest year per address),
        //         compute mean, stddev, and the z-score of the nearest point's price.
        let query = sqlx::query_as::<_, (f64, String, i64)>(
            r#"
            WITH point_zone AS (
                SELECT zone_type
                FROM zoning
                WHERE ST_Contains(geom, ST_SetSRID(ST_MakePoint($1, $2), 4326))
                LIMIT 1
            ),
            nearest_price AS (
                SELECT lp.price_per_sqm
                FROM land_prices lp
                WHERE ST_DWithin(lp.geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)
                ORDER BY ST_Distance(lp.geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography)
                LIMIT 1
            ),
            zone_prices AS (
                SELECT lp.price_per_sqm
                FROM land_prices lp
                INNER JOIN point_zone pz ON true
                INNER JOIN zoning z ON ST_Contains(z.geom, lp.geom)
                    AND z.zone_type = pz.zone_type
                WHERE lp.year = (
                    SELECT MAX(year) FROM land_prices
                )
            ),
            stats AS (
                SELECT AVG(price_per_sqm)    AS mean_price,
                       STDDEV(price_per_sqm) AS stddev_price,
                       COUNT(*)              AS sample_count
                FROM zone_prices
            )
            SELECT
                CASE
                    WHEN s.stddev_price IS NULL OR s.stddev_price = 0
                    THEN 0.0
                    ELSE (np.price_per_sqm - s.mean_price) / s.stddev_price
                END AS z_score,
                COALESCE(pz.zone_type, '') AS zone_type,
                s.sample_count
            FROM stats s
            CROSS JOIN nearest_price np
            LEFT JOIN point_zone pz ON true
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(z_score = row.0, zone_type = %row.1, sample_count = row.2, "price_z_score computed");

        Ok(ZScoreResult {
            z_score: row.0,
            zone_type: row.1,
            sample_count: row.2,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn count_recent_transactions(&self, coord: &Coord) -> Result<i64, DomainError> {
        // Count land_prices within 500m where year >= (max_year - 1).
        // This captures the latest full year and prior year for recency assessment.
        let query = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(*)
            FROM land_prices
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)
              AND year >= (SELECT MAX(year) - 1 FROM land_prices)
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;

        Ok(row.0)
    }
}

use async_trait::async_trait;
use realestate_db::spatial::bind_coord;
use sqlx::PgPool;

use super::map_db_err;
use crate::domain::entity::PriceRecord;
use crate::domain::error::DomainError;
use crate::domain::repository::ScoreRepository;
use crate::domain::value_object::Coord;

pub struct PgScoreRepository {
    pool: PgPool,
}

impl PgScoreRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ScoreRepository for PgScoreRepository {
    #[tracing::instrument(skip(self))]
    async fn find_nearest_prices(&self, coord: &Coord) -> Result<Vec<PriceRecord>, DomainError> {
        // Search radius: RADIUS_FACILITY_SEARCH_M (1000m), SRID: SRID_WGS84 (4326)
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
        tracing::debug!(row_count = rows.len(), "nearest_prices fetched");

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
    async fn calc_flood_overlap(&self, coord: &Coord) -> Result<f64, DomainError> {
        // Buffer radius: RADIUS_RISK_BUFFER_M (500m), SRID: SRID_WGS84 (4326)
        let query = sqlx::query_as::<_, (f64,)>(
            r#"
            SELECT COALESCE(
                SUM(ST_Area(ST_Intersection(
                    ST_Buffer(ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)::geometry,
                    geom
                ))) / NULLIF(ST_Area(ST_Buffer(ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)::geometry), 0),
                0.0
            )
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
        // Buffer radius: RADIUS_RISK_BUFFER_M (500m), SRID: SRID_WGS84 (4326)
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
    async fn count_schools_nearby(&self, coord: &Coord) -> Result<(i64, f64), DomainError> {
        // Search radius: RADIUS_FACILITY_SEARCH_M (1000m), SRID: SRID_WGS84 (4326)
        let query = sqlx::query_as::<_, (i64, Option<f64>)>(
            r#"
            SELECT COUNT(*),
                   MIN(ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography))
            FROM schools
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;

        Ok((row.0, row.1.unwrap_or(f64::MAX)))
    }

    #[tracing::instrument(skip(self))]
    async fn count_medical_nearby(&self, coord: &Coord) -> Result<(i64, f64), DomainError> {
        // Search radius: RADIUS_FACILITY_SEARCH_M (1000m), SRID: SRID_WGS84 (4326)
        let query = sqlx::query_as::<_, (i64, Option<f64>)>(
            r#"
            SELECT COUNT(*),
                   MIN(ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography))
            FROM medical_facilities
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)
            "#,
        );
        let row = bind_coord(query, coord.lng(), coord.lat())
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;

        Ok((row.0, row.1.unwrap_or(f64::MAX)))
    }
}

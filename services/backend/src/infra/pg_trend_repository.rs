use async_trait::async_trait;
use realestate_db::spatial::bind_coord;
use sqlx::PgPool;

use super::map_db_err;
use crate::domain::entity::{TrendLocation, TrendPoint};
use crate::domain::error::DomainError;
use crate::domain::repository::TrendRepository;
use crate::domain::value_object::{Coord, YearsLookback};

pub struct PgTrendRepository {
    pool: PgPool,
}

impl PgTrendRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TrendRepository for PgTrendRepository {
    #[tracing::instrument(skip(self))]
    async fn find_trend(
        &self,
        coord: Coord,
        years: YearsLookback,
    ) -> Result<Option<(TrendLocation, Vec<TrendPoint>)>, DomainError> {
        let years = years.value();
        // Search radius: RADIUS_TREND_SEARCH_M (2000m), SRID: SRID_WGS84 (4326)
        let nearest_query = sqlx::query_as::<_, (String, f64)>(
            r#"
            SELECT address,
                   ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS dist
            FROM land_prices
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 2000)
            ORDER BY dist
            LIMIT 1
            "#,
        );
        let nearest = bind_coord(nearest_query, coord.lng(), coord.lat())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(found = nearest.is_some(), "nearest_trend_point lookup");

        let Some((address, distance_m)) = nearest else {
            return Ok(None);
        };

        let max_year_row: (i32,) =
            sqlx::query_as("SELECT COALESCE(MAX(year), 0) FROM land_prices WHERE address = $1")
                .bind(&address)
                .fetch_one(&self.pool)
                .await
                .map_err(map_db_err)?;
        let min_year = max_year_row.0 - years + 1;

        let data = sqlx::query_as::<_, (i32, i32)>(
            r#"
            SELECT year, price_per_sqm
            FROM land_prices
            WHERE address = $1 AND year >= $2
            ORDER BY year
            "#,
        )
        .bind(&address)
        .bind(min_year)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)?;
        tracing::debug!(row_count = data.len(), "trend_data fetched");

        let points: Vec<TrendPoint> = data
            .into_iter()
            .map(|(year, price)| TrendPoint {
                year,
                price_per_sqm: price as i64,
            })
            .collect();

        Ok(Some((
            TrendLocation {
                address,
                distance_m,
            },
            points,
        )))
    }
}

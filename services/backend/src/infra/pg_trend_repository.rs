use std::time::Duration;

use async_trait::async_trait;
use realestate_db::spatial::bind_coord;
use sqlx::{FromRow, PgPool};
use tokio::time::timeout;

use super::map_db_err;
use crate::domain::entity::{TrendLocation, TrendPoint};
use crate::domain::error::DomainError;
use crate::domain::repository::TrendRepository;
use crate::domain::value_object::{Coord, YearsLookback};

/// Maximum time to wait for a single trend query.
const TREND_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, FromRow)]
struct NearestLocationRow {
    address: String,
    dist: f64,
}

#[derive(Debug, FromRow)]
struct TrendDataRow {
    survey_year: i32,
    price_per_sqm: i32,
}

impl From<TrendDataRow> for TrendPoint {
    fn from(row: TrendDataRow) -> Self {
        TrendPoint {
            year: row.survey_year,
            price_per_sqm: row.price_per_sqm as i64,
        }
    }
}

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
        let nearest_query = sqlx::query_as::<_, NearestLocationRow>(
            r#"
            SELECT address,
                   ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS dist
            FROM land_prices
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 2000)
            ORDER BY dist
            LIMIT 1
            "#,
        );
        let nearest = timeout(
            TREND_QUERY_TIMEOUT,
            bind_coord(nearest_query, coord.lng(), coord.lat()).fetch_optional(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("nearest_trend_point query".into()))?
        .map_err(map_db_err)
        .inspect(|n| tracing::debug!(found = n.is_some(), "nearest_trend_point lookup"))?;

        let Some(NearestLocationRow {
            address,
            dist: distance_m,
        }) = nearest
        else {
            return Ok(None);
        };

        let max_year_row: (i32,) = timeout(
            TREND_QUERY_TIMEOUT,
            sqlx::query_as(
                "SELECT COALESCE(MAX(survey_year::int), 0) FROM land_prices WHERE address = $1",
            )
            .bind(&address)
            .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("trend_max_year query".into()))?
        .map_err(map_db_err)?;
        let min_year = max_year_row.0 - years + 1;

        let data = timeout(
            TREND_QUERY_TIMEOUT,
            sqlx::query_as::<_, TrendDataRow>(
                r#"
            SELECT survey_year::int, price_per_sqm
            FROM land_prices
            WHERE address = $1 AND survey_year >= $2
            ORDER BY survey_year
            "#,
            )
            .bind(&address)
            .bind(min_year)
            .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("trend_data query".into()))?
        .map_err(map_db_err)
        .inspect(|rows| tracing::debug!(row_count = rows.len(), "trend_data fetched"))?;

        let points: Vec<TrendPoint> = data.into_iter().map(TrendPoint::from).collect();

        Ok(Some((
            TrendLocation {
                address,
                distance_m,
            },
            points,
        )))
    }
}

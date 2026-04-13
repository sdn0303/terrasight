use std::time::Duration;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use terrasight_server::db::spatial::bind_coord;
use tokio::time::timeout;

use super::map_db_err;
use crate::domain::constants::RADIUS_TREND_SEARCH_M;
use crate::domain::entity::{Address, PricePerSqm, TrendLocation, TrendPoint};
use crate::domain::error::DomainError;
use crate::domain::repository::TrendRepository;
use crate::domain::value_object::{Coord, Year, YearsLookback};

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

impl TryFrom<TrendDataRow> for TrendPoint {
    type Error = DomainError;

    fn try_from(row: TrendDataRow) -> Result<Self, Self::Error> {
        Ok(TrendPoint {
            year: Year::new(row.survey_year)?,
            price_per_sqm: PricePerSqm::new(i64::from(row.price_per_sqm))?,
        })
    }
}

#[derive(Debug, sqlx::FromRow)]
struct MaxYearRow {
    max_year: i32,
}

pub(crate) struct PgTrendRepository {
    pool: PgPool,
}

impl PgTrendRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
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
        // Search radius: RADIUS_TREND_SEARCH_M, SRID: SRID_WGS84 (4326)
        let nearest_query = sqlx::query_as::<_, NearestLocationRow>(
            r#"
            SELECT address,
                   ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS dist
            FROM land_prices
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
            ORDER BY dist
            LIMIT 1
            "#,
        );
        let nearest = timeout(
            TREND_QUERY_TIMEOUT,
            bind_coord(nearest_query, coord.lng(), coord.lat())
                .bind(RADIUS_TREND_SEARCH_M)
                .fetch_optional(&self.pool),
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

        let max_year_row: MaxYearRow = timeout(
            TREND_QUERY_TIMEOUT,
            sqlx::query_as(
                "SELECT COALESCE(MAX(survey_year::int), 0) AS max_year FROM land_prices WHERE address = $1",
            )
            .bind(&address)
            .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("trend_max_year query".into()))?
        .map_err(map_db_err)
        .inspect(|r: &MaxYearRow| tracing::debug!(max_year = r.max_year, "trend_max_year fetched"))?;
        let min_year = max_year_row.max_year - years + 1;

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

        let points: Vec<TrendPoint> = data
            .into_iter()
            .map(TrendPoint::try_from)
            .collect::<Result<_, _>>()?;

        Ok(Some((
            TrendLocation {
                address: Address::parse(&address)?,
                distance_m,
            },
            points,
        )))
    }
}

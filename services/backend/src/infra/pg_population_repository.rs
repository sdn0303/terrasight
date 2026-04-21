//! PostgreSQL implementation of [`PopulationRepository`].
//!
//! Implements [`PopulationRepository`](crate::domain::repository::PopulationRepository)
//! for the `GET /api/v1/population` endpoint. Queries the `mv_population_summary`
//! materialized view and LEFT JOINs `admin_boundaries` (level = `'municipality'`)
//! to resolve city names.
//!
//! ## SQL strategy
//!
//! Filters by `pref_code = $1`. The materialized view already pivots the four
//! population categories (`0010`, `0020`, `0030`, `0040`) into columns, so no
//! dynamic SQL is required. Results are ordered by `city_code`.

use std::time::Duration;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

use crate::domain::error::DomainError;
use crate::domain::model::{AreaName, CityCode, PopulationSummary, PrefCode};
use crate::domain::repository::PopulationRepository;
use crate::infra::query_helpers::run_query;

const POPULATION_QUERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Raw row from `mv_population_summary` joined with `admin_boundaries`.
#[derive(Debug, FromRow)]
struct PopulationRow {
    city_code: String,
    city_name: Option<String>,
    population: Option<i32>,
    male: Option<i32>,
    female: Option<i32>,
    households: Option<i32>,
    census_year: i16,
}

impl From<PopulationRow> for PopulationSummary {
    fn from(row: PopulationRow) -> Self {
        Self {
            city_code: CityCode::new(&row.city_code)
                .expect("INVARIANT: DB stores valid 5-digit city codes"),
            city_name: row
                .city_name
                .as_deref()
                .and_then(|n| AreaName::parse(n).ok())
                .unwrap_or_else(|| {
                    AreaName::parse(&row.city_code)
                        .expect("INVARIANT: city_code is non-empty fallback")
                }),
            population: row.population.unwrap_or(0),
            male: row.male,
            female: row.female,
            households: row.households,
            census_year: row.census_year,
        }
    }
}

/// PostgreSQL implementation of [`PopulationRepository`](crate::domain::repository::PopulationRepository).
pub struct PgPopulationRepository {
    pool: PgPool,
}

impl PgPopulationRepository {
    /// Create a new repository backed by the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PopulationRepository for PgPopulationRepository {
    /// Fetch population summaries from `mv_population_summary` for the given prefecture.
    ///
    /// City names are resolved via a LEFT JOIN on `admin_boundaries` at the
    /// `municipality` level. When no matching boundary row exists the city code
    /// is used as a fallback name.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on a PostgreSQL error.
    #[tracing::instrument(skip(self))]
    async fn find_by_pref_code(
        &self,
        pref_code: &PrefCode,
    ) -> Result<Vec<PopulationSummary>, DomainError> {
        let rows = run_query(
            POPULATION_QUERY_TIMEOUT,
            "population query",
            sqlx::query_as::<_, PopulationRow>(
                r#"
                SELECT
                    ps.city_code,
                    ab.name AS city_name,
                    ps.population,
                    ps.male,
                    ps.female,
                    ps.households,
                    ps.census_year
                FROM mv_population_summary ps
                LEFT JOIN admin_boundaries ab
                    ON ab.code = ps.city_code
                   AND ab.level = 'municipality'
                WHERE ps.pref_code = $1
                ORDER BY ps.city_code
                "#,
            )
            .bind(pref_code.as_str())
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| {
            tracing::debug!(
                count = rows.len(),
                pref_code = pref_code.as_str(),
                "population rows fetched"
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "population query failed"))?;

        Ok(rows.into_iter().map(PopulationSummary::from).collect())
    }
}

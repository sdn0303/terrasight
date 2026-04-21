//! PostgreSQL implementation of [`VacancyRepository`].
//!
//! Implements [`VacancyRepository`](crate::domain::repository::VacancyRepository)
//! for the `GET /api/v1/vacancy` endpoint. Queries the `mv_vacancy_summary`
//! materialized view and LEFT JOINs `admin_boundaries` (level = `'municipality'`)
//! to resolve city names.
//!
//! ## SQL strategy
//!
//! Filters by `pref_code = $1`. The materialized view already contains the
//! pre-computed `vacancy_rate_pct` column. Results are ordered by `city_code`.

use std::time::Duration;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

use crate::domain::error::DomainError;
use crate::domain::model::{AreaName, CityCode, PrefCode, VacancySummary};
use crate::domain::repository::VacancyRepository;
use crate::infra::query_helpers::run_query;

const VACANCY_QUERY_TIMEOUT: Duration = Duration::from_secs(10);

/// Raw row from `mv_vacancy_summary` joined with `admin_boundaries`.
#[derive(Debug, FromRow)]
struct VacancyRow {
    city_code: String,
    city_name: Option<String>,
    vacancy_count: i32,
    total_houses: Option<i32>,
    vacancy_rate_pct: Option<f64>,
    survey_year: i16,
}

impl From<VacancyRow> for VacancySummary {
    fn from(row: VacancyRow) -> Self {
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
            vacancy_count: row.vacancy_count,
            total_houses: row.total_houses,
            vacancy_rate_pct: row.vacancy_rate_pct,
            survey_year: row.survey_year,
        }
    }
}

/// PostgreSQL implementation of [`VacancyRepository`](crate::domain::repository::VacancyRepository).
pub struct PgVacancyRepository {
    pool: PgPool,
}

impl PgVacancyRepository {
    /// Create a new repository backed by the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl VacancyRepository for PgVacancyRepository {
    /// Fetch vacancy summaries from `mv_vacancy_summary` for the given prefecture.
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
    ) -> Result<Vec<VacancySummary>, DomainError> {
        let rows = run_query(
            VACANCY_QUERY_TIMEOUT,
            "vacancy query",
            sqlx::query_as::<_, VacancyRow>(
                r#"
                SELECT
                    vs.city_code,
                    ab.name AS city_name,
                    vs.vacancy_count,
                    vs.total_houses,
                    vs.vacancy_rate_pct,
                    vs.survey_year
                FROM mv_vacancy_summary vs
                LEFT JOIN admin_boundaries ab
                    ON ab.code = vs.city_code
                   AND ab.level = 'municipality'
                WHERE vs.pref_code = $1
                ORDER BY vs.city_code
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
                "vacancy rows fetched"
            )
        })
        .inspect_err(|e| tracing::warn!(error = %e, "vacancy query failed"))?;

        Ok(rows.into_iter().map(VacancySummary::from).collect())
    }
}

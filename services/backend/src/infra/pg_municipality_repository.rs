//! PostgreSQL implementation of [`MunicipalityRepository`].
//!
//! Implements [`MunicipalityRepository`](crate::domain::repository::MunicipalityRepository)
//! for the `/api/v1/municipalities` endpoint. Queries the `admin_boundaries` table
//! which stores both prefecture-level and municipality-level administrative
//! boundary rows.
//!
//! ## SQL strategy
//!
//! Filters to `level = 'municipality'` and `city_code IS NOT NULL` to exclude
//! prefecture-level rows. Uses `SELECT DISTINCT` to deduplicate rows that arise
//! from geometry partitioning (multi-polygon boundaries stored as separate rows).
//! Results are ordered by `city_code` for deterministic output.

use std::time::Duration;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

use crate::domain::error::DomainError;
use crate::domain::model::{AreaName, CityCode, Municipality, PrefCode};
use crate::domain::repository::MunicipalityRepository;
use crate::infra::query_helpers::run_query;

/// Maximum time to wait for the municipality list query.
const MUNICIPALITY_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/// Raw row returned by the `admin_boundaries` table.
///
/// `city_code` and `city_name` are nullable at the schema level because
/// `admin_boundaries` stores both prefecture-level and municipality-level rows.
/// The WHERE clause filters to `level = 'municipality'` so these columns are
/// always present, but we map through `Option` to satisfy the type system.
#[derive(Debug, FromRow)]
struct MunicipalityRow {
    city_code: Option<String>,
    city_name: Option<String>,
    pref_code: String,
}

/// PostgreSQL implementation of [`MunicipalityRepository`](crate::domain::repository::MunicipalityRepository).
pub struct PgMunicipalityRepository {
    pool: PgPool,
}

impl PgMunicipalityRepository {
    /// Create a new repository backed by the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MunicipalityRepository for PgMunicipalityRepository {
    /// Fetch all municipalities for the given prefecture from `admin_boundaries`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on a PostgreSQL error.
    ///
    /// Filters to `level = 'municipality'` rows where `city_code IS NOT NULL`.
    /// Uses `SELECT DISTINCT` to deduplicate geometry-partitioned rows.
    /// Results are ordered by `city_code`.
    #[tracing::instrument(skip(self))]
    async fn find_municipalities(
        &self,
        pref_code: &PrefCode,
    ) -> Result<Vec<Municipality>, DomainError> {
        let rows = run_query(
            MUNICIPALITY_QUERY_TIMEOUT,
            "municipalities query",
            sqlx::query_as::<_, MunicipalityRow>(
                r#"
            SELECT DISTINCT
                city_code,
                city_name,
                pref_code
            FROM admin_boundaries
            WHERE pref_code = $1
              AND level = 'municipality'
              AND city_code IS NOT NULL
            ORDER BY city_code
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
                "municipalities fetched"
            )
        })?;

        let municipalities = rows
            .into_iter()
            .filter_map(|row| {
                let city_code_str = row.city_code?;
                let city_name_str = row.city_name?;
                Some(Municipality {
                    city_code: CityCode::new(&city_code_str)
                        .expect("INVARIANT: DB stores valid city codes"),
                    city_name: AreaName::parse(&city_name_str)
                        .expect("INVARIANT: DB stores non-empty names"),
                    pref_code: PrefCode::new(&row.pref_code)
                        .expect("INVARIANT: DB stores valid pref codes"),
                })
            })
            .collect();

        Ok(municipalities)
    }
}

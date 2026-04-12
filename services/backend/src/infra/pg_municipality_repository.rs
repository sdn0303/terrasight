use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

use super::map_db_err;
use crate::domain::error::DomainError;
use crate::domain::municipality::Municipality;
use crate::domain::repository::MunicipalityRepository;
use crate::domain::value_object::PrefCode;

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

/// PostgreSQL implementation of [`MunicipalityRepository`].
pub struct PgMunicipalityRepository {
    pool: PgPool,
}

impl PgMunicipalityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MunicipalityRepository for PgMunicipalityRepository {
    /// Fetch all municipalities for the given prefecture from `admin_boundaries`.
    ///
    /// Filters to `level = 'municipality'` rows where `city_code IS NOT NULL`.
    /// Uses `SELECT DISTINCT` to deduplicate geometry-partitioned rows.
    /// Results are ordered by `city_code`.
    #[tracing::instrument(skip(self))]
    async fn find_municipalities(
        &self,
        pref_code: &PrefCode,
    ) -> Result<Vec<Municipality>, DomainError> {
        let rows = sqlx::query_as::<_, MunicipalityRow>(
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
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)
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
                let city_code = row.city_code?;
                let city_name = row.city_name?;
                Some(Municipality {
                    city_code,
                    city_name,
                    pref_code: row.pref_code,
                })
            })
            .collect();

        Ok(municipalities)
    }
}

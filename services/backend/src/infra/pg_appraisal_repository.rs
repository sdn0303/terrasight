//! PostgreSQL implementation of [`AppraisalRepository`].
//!
//! Implements [`AppraisalRepository`](crate::domain::repository::AppraisalRepository)
//! for the `/api/v1/appraisals` endpoint. Queries the `land_appraisals` table
//! which holds MLIT official appraisal records (`地価公示` / `地価調査`).
//!
//! ## SQL strategy
//!
//! Filters by `pref_code = $1` (required) and optionally `city_code = $2`
//! using the `$2::text IS NULL OR city_code = $2` pattern to avoid dynamic
//! SQL construction. Results are ordered by `city_code, price_per_sqm DESC`.

use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

use super::map_db_err;
use crate::domain::appraisal::AppraisalDetail;
use crate::domain::entity::{Address, AreaName, ZoneCode};
use crate::domain::error::DomainError;
use crate::domain::repository::AppraisalRepository;
use crate::domain::value_object::{CityCode, PrefCode};

/// Raw row returned by the `land_appraisals` table.
#[derive(Debug, FromRow)]
struct AppraisalDetailRow {
    city_code: String,
    city_name: String,
    address: String,
    land_use_code: String,
    price_per_sqm: i32,
    appraisal_price: i64,
    lot_area_sqm: Option<f32>,
    zone_code: Option<String>,
    building_coverage: Option<i16>,
    floor_area_ratio: Option<i16>,
    comparable_price: Option<i32>,
    yield_price: Option<i32>,
    cost_price: Option<i32>,
    fudosan_id: Option<String>,
}

impl From<AppraisalDetailRow> for AppraisalDetail {
    fn from(row: AppraisalDetailRow) -> Self {
        AppraisalDetail {
            city_code: CityCode::new(&row.city_code)
                .expect("INVARIANT: DB stores valid city codes"),
            city_name: AreaName::parse(&row.city_name)
                .expect("INVARIANT: DB stores non-empty names"),
            address: Address::parse(&row.address)
                .expect("INVARIANT: DB stores non-empty addresses"),
            land_use_code: row.land_use_code,
            price_per_sqm: row.price_per_sqm,
            appraisal_price: row.appraisal_price,
            lot_area_sqm: row.lot_area_sqm,
            zone_code: row
                .zone_code
                .as_deref()
                .map(|z| ZoneCode::parse(z).expect("INVARIANT: DB stores valid zone codes")),
            building_coverage: row.building_coverage,
            floor_area_ratio: row.floor_area_ratio,
            comparable_price: row.comparable_price,
            yield_price: row.yield_price,
            cost_price: row.cost_price,
            fudosan_id: row.fudosan_id,
        }
    }
}

/// PostgreSQL implementation of [`AppraisalRepository`](crate::domain::repository::AppraisalRepository).
pub struct PgAppraisalRepository {
    pool: PgPool,
}

impl PgAppraisalRepository {
    /// Create a new repository backed by the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AppraisalRepository for PgAppraisalRepository {
    /// Fetch appraisal records from `land_appraisals`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Database`] on a PostgreSQL error.
    ///
    /// Always filters by `pref_code`. Optional `city_code` narrows results to
    /// a single municipality using the `$2::text IS NULL OR city_code = $2` pattern.
    /// Results are ordered by `city_code, price_per_sqm DESC`.
    #[tracing::instrument(skip(self))]
    async fn find_appraisals(
        &self,
        pref_code: &PrefCode,
        city_code: Option<&CityCode>,
    ) -> Result<Vec<AppraisalDetail>, DomainError> {
        let rows = sqlx::query_as::<_, AppraisalDetailRow>(
            r#"
            SELECT
                city_code,
                city_name,
                address,
                land_use_code,
                price_per_sqm,
                appraisal_price,
                lot_area_sqm,
                zone_code,
                building_coverage,
                floor_area_ratio,
                comparable_price,
                yield_price,
                cost_price,
                fudosan_id
            FROM land_appraisals
            WHERE pref_code = $1
              AND ($2::text IS NULL OR city_code = $2)
            ORDER BY city_code, price_per_sqm DESC
            "#,
        )
        .bind(pref_code.as_str())
        .bind(city_code.map(CityCode::as_str))
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)
        .inspect(|rows| {
            tracing::debug!(
                count = rows.len(),
                pref_code = pref_code.as_str(),
                "appraisals fetched"
            )
        })?;

        Ok(rows.into_iter().map(AppraisalDetail::from).collect())
    }
}

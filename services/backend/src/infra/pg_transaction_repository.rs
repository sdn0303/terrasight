use async_trait::async_trait;
use sqlx::{FromRow, PgPool};

use super::map_db_err;
use crate::domain::error::DomainError;
use crate::domain::repository::TransactionRepository;
use crate::domain::transaction::{TransactionDetail, TransactionSummary};
use crate::domain::value_object::{PrefCode, Year};

/// Raw row returned by the `mv_transaction_summary` materialized view.
#[derive(Debug, FromRow)]
struct TransactionSummaryRow {
    city_code: String,
    transaction_year: i16,
    property_type: String,
    tx_count: i32,
    avg_total_price: i64,
    median_total_price: i64,
    avg_price_sqm: Option<i32>,
    avg_area_sqm: Option<i32>,
    avg_walk_min: Option<i16>,
}

impl From<TransactionSummaryRow> for TransactionSummary {
    fn from(row: TransactionSummaryRow) -> Self {
        TransactionSummary {
            city_code: row.city_code,
            transaction_year: row.transaction_year,
            property_type: row.property_type,
            tx_count: row.tx_count,
            avg_total_price: row.avg_total_price,
            median_total_price: row.median_total_price,
            avg_price_sqm: row.avg_price_sqm,
            avg_area_sqm: row.avg_area_sqm,
            avg_walk_min: row.avg_walk_min,
        }
    }
}

/// Raw row returned by the `transaction_prices` table.
#[derive(Debug, FromRow)]
struct TransactionDetailRow {
    city_code: String,
    city_name: String,
    district_name: Option<String>,
    property_type: String,
    total_price: i64,
    price_per_sqm: Option<i32>,
    area_sqm: Option<i32>,
    floor_plan: Option<String>,
    building_year: Option<i16>,
    building_structure: Option<String>,
    nearest_station: Option<String>,
    station_walk_min: Option<i16>,
    transaction_quarter: String,
}

impl From<TransactionDetailRow> for TransactionDetail {
    fn from(row: TransactionDetailRow) -> Self {
        TransactionDetail {
            city_code: row.city_code,
            city_name: row.city_name,
            district_name: row.district_name,
            property_type: row.property_type,
            total_price: row.total_price,
            price_per_sqm: row.price_per_sqm,
            area_sqm: row.area_sqm,
            floor_plan: row.floor_plan,
            building_year: row.building_year,
            building_structure: row.building_structure,
            nearest_station: row.nearest_station,
            station_walk_min: row.station_walk_min,
            transaction_quarter: row.transaction_quarter,
        }
    }
}

/// PostgreSQL implementation of [`TransactionRepository`].
pub struct PgTransactionRepository {
    pool: PgPool,
}

impl PgTransactionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TransactionRepository for PgTransactionRepository {
    /// Fetch aggregated transaction summaries from `mv_transaction_summary`.
    ///
    /// Optional `year_from` adds an `AND transaction_year >= $2` clause.
    /// Optional `property_type` adds an `AND property_type = $3` clause.
    /// Uses `$N::type IS NULL OR column = $N` pattern to avoid dynamic SQL.
    #[tracing::instrument(skip(self))]
    async fn find_transaction_summary(
        &self,
        pref_code: &PrefCode,
        year_from: Option<&Year>,
        property_type: Option<&str>,
    ) -> Result<Vec<TransactionSummary>, DomainError> {
        let rows = sqlx::query_as::<_, TransactionSummaryRow>(
            r#"
            SELECT
                city_code,
                transaction_year,
                property_type,
                tx_count,
                avg_total_price,
                median_total_price,
                avg_price_sqm,
                avg_area_sqm,
                avg_walk_min
            FROM mv_transaction_summary
            WHERE pref_code = $1
              AND ($2::smallint IS NULL OR transaction_year >= $2)
              AND ($3::text IS NULL OR property_type = $3)
            ORDER BY city_code, transaction_year DESC
            "#,
        )
        .bind(pref_code.as_str())
        .bind(year_from.map(|y| y.value() as i16))
        .bind(property_type)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)
        .inspect(|rows| {
            tracing::debug!(
                count = rows.len(),
                pref_code = pref_code.as_str(),
                "transaction_summary fetched"
            )
        })?;

        Ok(rows.into_iter().map(TransactionSummary::from).collect())
    }

    /// Fetch individual transaction records for a given city code.
    ///
    /// Optional `year_from` restricts results to records on or after that year.
    /// Results are ordered by `transaction_year DESC, transaction_q DESC` and
    /// capped to `limit` rows.
    #[tracing::instrument(skip(self))]
    async fn find_transactions(
        &self,
        city_code: &str,
        year_from: Option<&Year>,
        limit: u32,
    ) -> Result<Vec<TransactionDetail>, DomainError> {
        let rows = sqlx::query_as::<_, TransactionDetailRow>(
            r#"
            SELECT
                city_code,
                city_name,
                district_name,
                property_type,
                total_price,
                price_per_sqm,
                area_sqm,
                floor_plan,
                building_year,
                building_structure,
                nearest_station,
                station_walk_min,
                transaction_quarter
            FROM transaction_prices
            WHERE city_code = $1
              AND ($2::smallint IS NULL OR transaction_year >= $2)
            ORDER BY transaction_year DESC, transaction_q DESC
            LIMIT $3
            "#,
        )
        .bind(city_code)
        .bind(year_from.map(|y| y.value() as i16))
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_err)
        .inspect(|rows| {
            tracing::debug!(count = rows.len(), city_code, limit, "transactions fetched")
        })?;

        Ok(rows.into_iter().map(TransactionDetail::from).collect())
    }
}

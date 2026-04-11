use std::time::Duration;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use tokio::time::timeout;

use super::map_db_err;
use crate::domain::entity::{AdminAreaStats, FacilityStats, LandPriceStats, RiskStats};
use crate::domain::error::DomainError;
use crate::domain::repository::AdminAreaStatsRepository;
use crate::domain::value_object::{AreaCode, AreaCodeLevel};

/// Maximum time to wait for any admin-area stats query.
const ADMIN_STATS_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, FromRow)]
struct AdminLandPriceStatsRow {
    avg_price: Option<f64>,
    median_price: Option<f64>,
    min_price: Option<i64>,
    max_price: Option<i64>,
    count: i64,
}

impl From<AdminLandPriceStatsRow> for LandPriceStats {
    fn from(row: AdminLandPriceStatsRow) -> Self {
        LandPriceStats {
            avg_per_sqm: row.avg_price,
            median_per_sqm: row.median_price,
            min_per_sqm: row.min_price,
            max_per_sqm: row.max_price,
            count: row.count,
        }
    }
}

pub struct PgAdminAreaStatsRepository {
    pool: PgPool,
}

impl PgAdminAreaStatsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AdminAreaStatsRepository for PgAdminAreaStatsRepository {
    /// Fetch aggregated statistics for the given administrative area code.
    ///
    /// The current implementation returns global land-price aggregates because the
    /// `admin_boundaries` PostGIS table (populated by the Phase 5 data pipeline) does
    /// not exist yet.  Once that table is available the queries should be narrowed to
    /// `WHERE ST_Intersects(geom, (SELECT geom FROM admin_boundaries WHERE code = $1))`.
    #[tracing::instrument(skip(self))]
    async fn get_area_stats(&self, code: &AreaCode) -> Result<AdminAreaStats, DomainError> {
        let level = match code.level() {
            AreaCodeLevel::Prefecture => "prefecture",
            AreaCodeLevel::Municipality => "municipality",
        };

        // Land price stats — global aggregate (placeholder until admin_boundaries exists).
        let lp_row = timeout(
            ADMIN_STATS_QUERY_TIMEOUT,
            sqlx::query_as::<_, AdminLandPriceStatsRow>(
                r#"
            SELECT
                AVG(price_per_sqm)::float8 AS avg_price,
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY price_per_sqm)::float8 AS median_price,
                MIN(price_per_sqm)::int8 AS min_price,
                MAX(price_per_sqm)::int8 AS max_price,
                COUNT(*) AS count
            FROM land_prices
            WHERE year = (SELECT MAX(year) FROM land_prices)
            "#,
            )
            .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("admin_area land_price_stats query".into()))?
        .map_err(map_db_err)
        .inspect(|row| tracing::debug!(count = row.count, "admin_area land_price_stats fetched"))?;

        // Facility counts — global aggregate (placeholder until admin_boundaries exists).
        let schools_row = timeout(
            ADMIN_STATS_QUERY_TIMEOUT,
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM schools").fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("admin_area schools_count query".into()))?
        .map_err(map_db_err)?;

        let medical_row = timeout(
            ADMIN_STATS_QUERY_TIMEOUT,
            sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM medical_facilities")
                .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("admin_area medical_count query".into()))?
        .map_err(map_db_err)?;

        tracing::debug!(
            schools = schools_row.0,
            medical = medical_row.0,
            "admin_area facility_counts fetched"
        );

        Ok(AdminAreaStats {
            code: code.as_str().to_string(),
            // Placeholder name until admin_boundaries table is populated.
            name: format!("Area {}", code.as_str()),
            level: level.to_string(),
            land_price: lp_row.into(),
            risk: RiskStats {
                flood_area_ratio: 0.0,
                steep_slope_area_ratio: 0.0,
                composite_risk: 0.0,
            },
            facilities: FacilityStats {
                schools: schools_row.0,
                medical: medical_row.0,
            },
        })
    }
}

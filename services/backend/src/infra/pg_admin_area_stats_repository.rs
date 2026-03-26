use async_trait::async_trait;
use sqlx::PgPool;

use super::map_db_err;
use crate::domain::entity::{AdminAreaStats, FacilityStats, LandPriceStats, RiskStats};
use crate::domain::error::DomainError;
use crate::domain::repository::AdminAreaStatsRepository;

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
    async fn get_area_stats(&self, code: &str) -> Result<AdminAreaStats, DomainError> {
        let level = if code.len() <= 2 {
            "prefecture"
        } else {
            "municipality"
        };

        // Land price stats — global aggregate (placeholder until admin_boundaries exists).
        let lp_row =
            sqlx::query_as::<_, (Option<f64>, Option<f64>, Option<i64>, Option<i64>, i64)>(
                r#"
            SELECT
                AVG(price_per_sqm)::float8,
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY price_per_sqm)::float8,
                MIN(price_per_sqm)::int8,
                MAX(price_per_sqm)::int8,
                COUNT(*)
            FROM land_prices
            WHERE year = (SELECT MAX(year) FROM land_prices)
            "#,
            )
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;

        tracing::debug!(count = lp_row.4, "admin_area land_price_stats fetched");

        // Facility counts — global aggregate (placeholder until admin_boundaries exists).
        let schools_row = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM schools")
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;

        let medical_row = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM medical_facilities")
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_err)?;

        tracing::debug!(
            schools = schools_row.0,
            medical = medical_row.0,
            "admin_area facility_counts fetched"
        );

        Ok(AdminAreaStats {
            code: code.to_string(),
            // Placeholder name until admin_boundaries table is populated.
            name: format!("Area {code}"),
            level: level.to_string(),
            land_price: LandPriceStats {
                avg_per_sqm: lp_row.0,
                median_per_sqm: lp_row.1,
                min_per_sqm: lp_row.2,
                max_per_sqm: lp_row.3,
                count: lp_row.4,
            },
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

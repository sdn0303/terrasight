//! PostgreSQL implementation of [`AdminAreaStatsRepository`].
//!
//! Implements [`AdminAreaStatsRepository`](crate::domain::repository::AdminAreaStatsRepository)
//! which serves the `/api/v1/area-stats` endpoint.
//!
//! ## Current state (placeholder)
//!
//! The implementation currently returns **global** aggregates because the
//! `admin_boundaries` PostGIS table (populated by the Phase 5 data pipeline)
//! does not yet exist. Once that table is available, queries should be narrowed
//! with a `WHERE ST_Intersects(geom, (SELECT geom FROM admin_boundaries WHERE code = $1))`
//! predicate so results reflect only the requested administrative area.
//!
//! Risk stats (flood ratio, slope ratio, composite) are returned as zeros for
//! the same reason — the spatial join against hazard layers cannot be scoped
//! until the boundary geometry is present.
//!
//! All queries enforce [`ADMIN_STATS_QUERY_TIMEOUT`] via
//! [`run_query`](crate::infra::query_helpers::run_query).

use std::time::Duration;

use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::error::DomainError;
use crate::domain::model::{
    AdminAreaStats, AreaCode, AreaCodeLevel, AreaName, FacilityStats, RiskStats,
};
use crate::domain::repository::AdminAreaStatsRepository;
use crate::infra::query_helpers::run_query;
use crate::infra::row_types::{CountRow, LandPriceStatsRow};

/// Maximum time to wait for any admin-area stats query.
const ADMIN_STATS_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/// PostgreSQL implementation of [`AdminAreaStatsRepository`](crate::domain::repository::AdminAreaStatsRepository).
pub(crate) struct PgAdminAreaStatsRepository {
    pool: PgPool,
}

impl PgAdminAreaStatsRepository {
    /// Create a new repository backed by the given connection pool.
    pub(crate) fn new(pool: PgPool) -> Self {
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
        let lp_row = run_query(
            ADMIN_STATS_QUERY_TIMEOUT,
            "admin_area land_price_stats query",
            sqlx::query_as::<_, LandPriceStatsRow>(
                r#"
            SELECT
                AVG(price_per_sqm)::float8 AS avg_price,
                PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY price_per_sqm)::float8 AS median_price,
                MIN(price_per_sqm)::int8 AS min_price,
                MAX(price_per_sqm)::int8 AS max_price,
                COUNT(*) AS count
            FROM land_prices
            WHERE survey_year = (SELECT MAX(survey_year) FROM land_prices)
            "#,
            )
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|row| tracing::debug!(count = row.count, "admin_area land_price_stats fetched"))?;

        // Facility counts — global aggregate (placeholder until admin_boundaries exists).
        let schools_row = run_query(
            ADMIN_STATS_QUERY_TIMEOUT,
            "admin_area schools_count query",
            sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM schools")
                .fetch_one(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(count = r.count, "admin_area schools_count fetched"))?;

        let medical_row = run_query(
            ADMIN_STATS_QUERY_TIMEOUT,
            "admin_area medical_count query",
            sqlx::query_as::<_, CountRow>("SELECT COUNT(*) AS count FROM medical_facilities")
                .fetch_one(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(count = r.count, "admin_area medical_count fetched"))?;

        tracing::debug!(
            schools = schools_row.count,
            medical = medical_row.count,
            "admin_area facility_counts fetched"
        );

        Ok(AdminAreaStats {
            code: code.clone(),
            // Placeholder name until admin_boundaries table is populated.
            name: AreaName::parse(&format!("Area {}", code.as_str()))
                .expect("INVARIANT: placeholder area name is non-empty"),
            level: level.to_string(),
            land_price: lp_row.into(),
            risk: RiskStats {
                flood_area_ratio: 0.0,
                steep_slope_area_ratio: 0.0,
                composite_risk: 0.0,
            },
            facilities: FacilityStats {
                schools: schools_row.count,
                medical: medical_row.count,
            },
        })
    }
}

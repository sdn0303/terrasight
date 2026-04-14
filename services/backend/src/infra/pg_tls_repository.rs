//! PostgreSQL + PostGIS implementation of [`TlsRepository`].
//!
//! Implements [`TlsRepository`](crate::domain::repository::TlsRepository),
//! which supplies all PostGIS-sourced inputs for the Total Location Score (TLS)
//! computation. Every method issues a proximity search around a coordinate
//! using `ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1,$2),4326)::geography, $3)`.
//!
//! ## Query patterns
//!
//! | Method | PostGIS pattern | Radius constant |
//! |--------|----------------|-----------------|
//! | `find_nearest_prices` | CTE nearest-address join | [`TLS_PRICE_SEARCH_RADIUS_M`](crate::domain::constants::TLS_PRICE_SEARCH_RADIUS_M) |
//! | `find_flood_depth_rank` | `MAX(depth_rank::int)` | [`TLS_RISK_SEARCH_RADIUS_M`](crate::domain::constants::TLS_RISK_SEARCH_RADIUS_M) |
//! | `has_steep_slope_nearby` | `EXISTS(SELECT 1 …)` | [`TLS_RISK_SEARCH_RADIUS_M`](crate::domain::constants::TLS_RISK_SEARCH_RADIUS_M) |
//! | `find_schools_nearby` | `COUNT(*)` + `bool_or` | [`TLS_SCHOOL_SEARCH_RADIUS_M`](crate::domain::constants::TLS_SCHOOL_SEARCH_RADIUS_M) |
//! | `find_medical_nearby` | `COUNT(*) FILTER (WHERE …)` | [`TLS_PRICE_SEARCH_RADIUS_M`](crate::domain::constants::TLS_PRICE_SEARCH_RADIUS_M) |
//! | `find_zoning_far` | `ST_Contains` point-in-polygon | — |
//! | `calc_price_z_score` | CTE nearest + zone_stats z-score | [`TLS_PRICE_SEARCH_RADIUS_M`](crate::domain::constants::TLS_PRICE_SEARCH_RADIUS_M) |
//! | `count_recent_transactions` | `survey_year >= max_year - 1` | [`TLS_TRANSACTION_SEARCH_RADIUS_M`](crate::domain::constants::TLS_TRANSACTION_SEARCH_RADIUS_M) |
//!
//! All queries share [`TLS_QUERY_TIMEOUT`] enforced by
//! [`run_query`](crate::infra::query_helpers::run_query).

use std::time::Duration;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use terrasight_geo::GeoCoord;
use terrasight_server::db::spatial::bind_coord;

use crate::domain::constants::{
    TLS_PRICE_SEARCH_RADIUS_M, TLS_RISK_SEARCH_RADIUS_M, TLS_SCHOOL_SEARCH_RADIUS_M,
    TLS_TRANSACTION_SEARCH_RADIUS_M,
};
use crate::domain::entity::{MedicalStats, PriceRecord, SchoolStats, ZScoreResult};
use crate::domain::error::DomainError;
use crate::domain::repository::TlsRepository;
use crate::domain::value_object::Coord;
use crate::infra::query_helpers::run_query;
use crate::infra::row_types::CountRow;

/// Maximum time to wait for a single TLS sub-query.
///
/// 30 s gives local Docker-based integration tests enough headroom for
/// spatial queries (medical, school, flood, seismic) against imported data
/// sets of several thousand rows.  Production deployments with a co-located
/// DB and appropriate spatial indexes complete well within this budget.
const TLS_QUERY_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, FromRow)]
struct NearestPriceRow {
    year: i32,
    price_per_sqm: i32,
}

impl From<NearestPriceRow> for PriceRecord {
    fn from(row: NearestPriceRow) -> Self {
        PriceRecord {
            year: row.year,
            price_per_sqm: i64::from(row.price_per_sqm),
        }
    }
}

#[derive(Debug, FromRow)]
struct SchoolsNearbyRow {
    count: i64,
    has_primary: bool,
    has_junior_high: bool,
}

impl From<SchoolsNearbyRow> for SchoolStats {
    fn from(row: SchoolsNearbyRow) -> Self {
        SchoolStats {
            count_800m: row.count,
            has_primary: row.has_primary,
            has_junior_high: row.has_junior_high,
        }
    }
}

#[derive(Debug, FromRow)]
struct MedicalNearbyRow {
    hospital_count: i64,
    clinic_count: i64,
    total_beds: i64,
}

impl From<MedicalNearbyRow> for MedicalStats {
    fn from(row: MedicalNearbyRow) -> Self {
        MedicalStats {
            hospital_count: row.hospital_count,
            clinic_count: row.clinic_count,
            total_beds: row.total_beds,
        }
    }
}

#[derive(Debug, FromRow)]
struct ZScoreRow {
    z_score: f64,
    zone_type: String,
    sample_count: i64,
}

impl From<ZScoreRow> for ZScoreResult {
    fn from(row: ZScoreRow) -> Self {
        ZScoreResult {
            z_score: row.z_score,
            zone_type: row.zone_type,
            sample_count: row.sample_count,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct DepthRankRow {
    depth_rank: Option<i32>,
}

#[derive(Debug, sqlx::FromRow)]
struct BoolRow {
    exists: bool,
}

#[derive(Debug, sqlx::FromRow)]
struct OptionalF64Row {
    value: Option<f64>,
}

/// PostgreSQL + PostGIS implementation of [`TlsRepository`](crate::domain::repository::TlsRepository).
pub(crate) struct PgTlsRepository {
    pool: PgPool,
}

impl PgTlsRepository {
    /// Create a new repository backed by the given connection pool.
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TlsRepository for PgTlsRepository {
    /// Fetch all historical price records for the nearest land-price observation point.
    ///
    /// Uses a two-step CTE: first, find the closest address within
    /// [`TLS_PRICE_SEARCH_RADIUS_M`](crate::domain::constants::TLS_PRICE_SEARCH_RADIUS_M);
    /// then, return all yearly records for that address ordered by `survey_year`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn find_nearest_prices(&self, coord: &Coord) -> Result<Vec<PriceRecord>, DomainError> {
        // Search radius: TLS_PRICE_SEARCH_RADIUS_M, SRID: 4326
        let geo_coord = GeoCoord {
            lng: coord.lng(),
            lat: coord.lat(),
        };
        let rows = run_query(
            TLS_QUERY_TIMEOUT,
            "tls nearest_prices query",
            bind_coord(
                sqlx::query_as::<_, NearestPriceRow>(
                    r#"
            WITH nearest AS (
                SELECT address,
                       ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS dist
                FROM land_prices
                WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
                ORDER BY dist
                LIMIT 1
            )
            SELECT lp.survey_year::int AS year, lp.price_per_sqm
            FROM land_prices lp
            INNER JOIN nearest n ON lp.address = n.address
            ORDER BY lp.survey_year
            "#,
                ),
                &geo_coord,
            )
            .bind(TLS_PRICE_SEARCH_RADIUS_M)
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| tracing::debug!(row_count = rows.len(), "tls nearest_prices fetched"))?;

        Ok(rows.into_iter().map(PriceRecord::from).collect())
    }

    /// Return the maximum flood depth rank within the risk search radius, or `None` if no
    /// flood zone intersects.
    ///
    /// `depth_rank` is stored as `text`; numeric values are extracted with a safe
    /// `CASE WHEN depth_rank ~ '^\d+$' THEN depth_rank::int END` cast.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn find_flood_depth_rank(&self, coord: &Coord) -> Result<Option<i32>, DomainError> {
        // MAX depth_rank within TLS_RISK_SEARCH_RADIUS_M buffer. Returns NULL when no flood zone intersects.
        // depth_rank is text in the schema; safe cast ignores non-numeric values.
        let geo_coord = GeoCoord {
            lng: coord.lng(),
            lat: coord.lat(),
        };
        let row = run_query(
            TLS_QUERY_TIMEOUT,
            "tls flood_depth_rank query",
            bind_coord(
                sqlx::query_as::<_, DepthRankRow>(
                    r#"
            SELECT MAX(CASE WHEN depth_rank ~ '^\d+$' THEN depth_rank::int END) AS depth_rank
            FROM flood_risk
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
            "#,
                ),
                &geo_coord,
            )
            .bind(TLS_RISK_SEARCH_RADIUS_M)
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(depth_rank = ?r.depth_rank, "tls flood_depth_rank fetched"))?;

        Ok(row.depth_rank)
    }

    /// Return `true` if any steep-slope hazard polygon falls within the risk search radius.
    ///
    /// Uses `EXISTS(SELECT 1 …)` to short-circuit on the first matching row.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn has_steep_slope_nearby(&self, coord: &Coord) -> Result<bool, DomainError> {
        // TLS_RISK_SEARCH_RADIUS_M buffer, SRID: 4326
        // Uses EXISTS instead of COUNT to short-circuit on first match.
        let geo_coord = GeoCoord {
            lng: coord.lng(),
            lat: coord.lat(),
        };
        let row = run_query(
            TLS_QUERY_TIMEOUT,
            "tls steep_slope_nearby query",
            bind_coord(
                sqlx::query_as::<_, BoolRow>(
                    r#"
            SELECT EXISTS (
                SELECT 1
                FROM steep_slope
                WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
            ) AS exists
            "#,
                ),
                &geo_coord,
            )
            .bind(TLS_RISK_SEARCH_RADIUS_M)
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(exists = r.exists, "tls steep_slope_nearby fetched"))?;

        Ok(row.exists)
    }

    /// Count schools within the school search radius and detect primary / junior-high presence.
    ///
    /// Uses `COUNT(*)` and `bool_or(school_type = '小学校' | '中学校')` in a single query.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn find_schools_nearby(&self, coord: &Coord) -> Result<SchoolStats, DomainError> {
        // TLS_SCHOOL_SEARCH_RADIUS_M radius, SRID: 4326
        let geo_coord = GeoCoord {
            lng: coord.lng(),
            lat: coord.lat(),
        };
        let row = run_query(
            TLS_QUERY_TIMEOUT,
            "tls schools_nearby query",
            bind_coord(
                sqlx::query_as::<_, SchoolsNearbyRow>(
                    r#"
            SELECT COUNT(*) AS count,
                   COALESCE(bool_or(school_type = '小学校'), false) AS has_primary,
                   COALESCE(bool_or(school_type = '中学校'), false) AS has_junior_high
            FROM schools
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
            "#,
                ),
                &geo_coord,
            )
            .bind(TLS_SCHOOL_SEARCH_RADIUS_M)
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|row| {
            tracing::debug!(
                count = row.count,
                has_primary = row.has_primary,
                has_junior_high = row.has_junior_high,
                "schools_nearby fetched"
            )
        })?;

        Ok(row.into())
    }

    /// Count hospitals and clinics within the price search radius, summing bed counts.
    ///
    /// Splits facility types with `COUNT(*) FILTER (WHERE facility_type = '病院')` for hospitals
    /// and `!= '病院'` for clinics in a single aggregate query.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn find_medical_nearby(&self, coord: &Coord) -> Result<MedicalStats, DomainError> {
        // TLS_PRICE_SEARCH_RADIUS_M radius, SRID: 4326
        let geo_coord = GeoCoord {
            lng: coord.lng(),
            lat: coord.lat(),
        };
        let row = run_query(
            TLS_QUERY_TIMEOUT,
            "tls medical_nearby query",
            bind_coord(
                sqlx::query_as::<_, MedicalNearbyRow>(
                    r#"
            SELECT COUNT(*) FILTER (WHERE facility_type = '病院') AS hospital_count,
                   COUNT(*) FILTER (WHERE facility_type != '病院') AS clinic_count,
                   COALESCE(SUM(beds), 0)::int8 AS total_beds
            FROM medical_facilities
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
            "#,
                ),
                &geo_coord,
            )
            .bind(TLS_PRICE_SEARCH_RADIUS_M)
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|row| {
            tracing::debug!(
                hospitals = row.hospital_count,
                clinics = row.clinic_count,
                beds = row.total_beds,
                "medical_nearby fetched"
            )
        })?;

        Ok(row.into())
    }

    /// Return the floor-area ratio of the zoning polygon that contains `coord`, or `None`.
    ///
    /// Uses `ST_Contains(geom, ST_SetSRID(ST_MakePoint($1,$2), 4326))` for a point-in-polygon
    /// lookup. Returns `None` when no zoning polygon covers the coordinate.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn find_zoning_far(&self, coord: &Coord) -> Result<Option<f64>, DomainError> {
        // Find the zoning polygon that contains the point; return its floor_area_ratio.
        let geo_coord = GeoCoord {
            lng: coord.lng(),
            lat: coord.lat(),
        };
        let row = run_query(
            TLS_QUERY_TIMEOUT,
            "tls zoning_far query",
            bind_coord(
                sqlx::query_as::<_, OptionalF64Row>(
                    r#"
            SELECT floor_area_ratio::double precision AS value
            FROM zoning
            WHERE ST_Contains(geom, ST_SetSRID(ST_MakePoint($1, $2), 4326))
            LIMIT 1
            "#,
                ),
                &geo_coord,
            )
            .fetch_optional(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(found = r.is_some(), "tls zoning_far fetched"))?;

        Ok(row.and_then(|r| r.value))
    }

    /// Compute the price z-score of the nearest land-price point relative to its zone cohort.
    ///
    /// Uses a two-CTE approach: `nearest` finds the closest point within the price search
    /// radius; `zone_stats` computes mean and stddev across the zone in the latest year.
    /// Uses the denormalised `zone_type` column on `land_prices` (avoids a slow
    /// `ST_Contains` join against the `zoning` table that caused 503 errors in earlier
    /// revisions).
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn calc_price_z_score(&self, coord: &Coord) -> Result<ZScoreResult, DomainError> {
        // Uses the denormalized zone_type column on land_prices to avoid the slow
        // ST_Contains join against the zoning table that was causing 503 errors.
        let geo_coord = GeoCoord {
            lng: coord.lng(),
            lat: coord.lat(),
        };
        let row = run_query(
            TLS_QUERY_TIMEOUT,
            "tls price_z_score query",
            bind_coord(
                sqlx::query_as::<_, ZScoreRow>(
                    r#"
            WITH nearest AS (
                SELECT price_per_sqm, zone_type
                FROM land_prices
                WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
                ORDER BY ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography)
                LIMIT 1
            ),
            zone_stats AS (
                SELECT AVG(lp.price_per_sqm)::double precision AS mean_price,
                       STDDEV(lp.price_per_sqm)::double precision AS stddev_price,
                       COUNT(*)::bigint AS sample_count
                FROM land_prices lp, nearest n
                WHERE lp.zone_type = n.zone_type
                  AND lp.survey_year = (SELECT MAX(survey_year) FROM land_prices)
            )
            SELECT
                COALESCE(
                    CASE WHEN zs.stddev_price IS NULL OR zs.stddev_price = 0 THEN 0.0
                         ELSE ((n.price_per_sqm - zs.mean_price) / zs.stddev_price)
                    END, 0.0)::double precision AS z_score,
                COALESCE(n.zone_type, '') AS zone_type,
                COALESCE(zs.sample_count, 0)::bigint AS sample_count
            FROM nearest n
            LEFT JOIN zone_stats zs ON true
            "#,
                ),
                &geo_coord,
            )
            .bind(TLS_PRICE_SEARCH_RADIUS_M)
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|row| {
            tracing::debug!(
                z_score = row.z_score,
                zone_type = %row.zone_type,
                sample_count = row.sample_count,
                "price_z_score computed"
            )
        })?;

        Ok(row.into())
    }

    /// Count land-price records within the transaction search radius from the latest two survey years.
    ///
    /// Filters `survey_year >= (SELECT MAX(survey_year) - 1 FROM land_prices)` to capture
    /// the two most-recent complete years, giving a recency signal without a hard-coded year.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] or [`DomainError::Database`].
    #[tracing::instrument(skip(self))]
    async fn count_recent_transactions(&self, coord: &Coord) -> Result<i64, DomainError> {
        // Count land_prices within TLS_TRANSACTION_SEARCH_RADIUS_M where year >= (max_year - 1).
        // This captures the latest full year and prior year for recency assessment.
        let geo_coord = GeoCoord {
            lng: coord.lng(),
            lat: coord.lat(),
        };
        let row = run_query(
            TLS_QUERY_TIMEOUT,
            "tls recent_transactions query",
            bind_coord(
                sqlx::query_as::<_, CountRow>(
                    r#"
            SELECT COUNT(*) AS count
            FROM land_prices
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
              AND survey_year >= (SELECT MAX(survey_year) - 1 FROM land_prices)
            "#,
                ),
                &geo_coord,
            )
            .bind(TLS_TRANSACTION_SEARCH_RADIUS_M)
            .fetch_one(&self.pool),
        )
        .await
        .inspect(|r| tracing::debug!(count = r.count, "tls recent_transactions fetched"))?;

        Ok(row.count)
    }
}

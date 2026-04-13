use std::time::Duration;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool};
use terrasight_server::db::spatial::bind_coord;
use tokio::time::timeout;

use super::map_db_err;
use crate::domain::constants::{
    TLS_PRICE_SEARCH_RADIUS_M, TLS_RISK_SEARCH_RADIUS_M, TLS_SCHOOL_SEARCH_RADIUS_M,
    TLS_TRANSACTION_SEARCH_RADIUS_M,
};
use crate::domain::entity::{MedicalStats, PriceRecord, SchoolStats, ZScoreResult};
use crate::domain::error::DomainError;
use crate::domain::repository::TlsRepository;
use crate::domain::value_object::Coord;
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

pub(crate) struct PgTlsRepository {
    pool: PgPool,
}

impl PgTlsRepository {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TlsRepository for PgTlsRepository {
    #[tracing::instrument(skip(self))]
    async fn find_nearest_prices(&self, coord: &Coord) -> Result<Vec<PriceRecord>, DomainError> {
        // Search radius: TLS_PRICE_SEARCH_RADIUS_M, SRID: 4326
        let query = sqlx::query_as::<_, NearestPriceRow>(
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
        );
        let rows = timeout(
            TLS_QUERY_TIMEOUT,
            bind_coord(query, coord.lng(), coord.lat())
                .bind(TLS_PRICE_SEARCH_RADIUS_M)
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("tls nearest_prices query".into()))?
        .map_err(map_db_err)
        .inspect(|rows| tracing::debug!(row_count = rows.len(), "tls nearest_prices fetched"))?;

        Ok(rows.into_iter().map(PriceRecord::from).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn find_flood_depth_rank(&self, coord: &Coord) -> Result<Option<i32>, DomainError> {
        // MAX depth_rank within TLS_RISK_SEARCH_RADIUS_M buffer. Returns NULL when no flood zone intersects.
        // depth_rank is text in the schema; safe cast ignores non-numeric values.
        let query = sqlx::query_as::<_, DepthRankRow>(
            r#"
            SELECT MAX(CASE WHEN depth_rank ~ '^\d+$' THEN depth_rank::int END) AS depth_rank
            FROM flood_risk
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
            "#,
        );
        let row = timeout(
            TLS_QUERY_TIMEOUT,
            bind_coord(query, coord.lng(), coord.lat())
                .bind(TLS_RISK_SEARCH_RADIUS_M)
                .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("tls flood_depth_rank query".into()))?
        .map_err(map_db_err)
        .inspect(|r| tracing::debug!(depth_rank = ?r.depth_rank, "tls flood_depth_rank fetched"))?;

        Ok(row.depth_rank)
    }

    #[tracing::instrument(skip(self))]
    async fn has_steep_slope_nearby(&self, coord: &Coord) -> Result<bool, DomainError> {
        // TLS_RISK_SEARCH_RADIUS_M buffer, SRID: 4326
        // Uses EXISTS instead of COUNT to short-circuit on first match.
        let query = sqlx::query_as::<_, BoolRow>(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM steep_slope
                WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
            ) AS exists
            "#,
        );
        let row = timeout(
            TLS_QUERY_TIMEOUT,
            bind_coord(query, coord.lng(), coord.lat())
                .bind(TLS_RISK_SEARCH_RADIUS_M)
                .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("tls steep_slope_nearby query".into()))?
        .map_err(map_db_err)
        .inspect(|r| tracing::debug!(exists = r.exists, "tls steep_slope_nearby fetched"))?;

        Ok(row.exists)
    }

    #[tracing::instrument(skip(self))]
    async fn find_schools_nearby(&self, coord: &Coord) -> Result<SchoolStats, DomainError> {
        // TLS_SCHOOL_SEARCH_RADIUS_M radius, SRID: 4326
        let query = sqlx::query_as::<_, SchoolsNearbyRow>(
            r#"
            SELECT COUNT(*) AS count,
                   COALESCE(bool_or(school_type = '小学校'), false) AS has_primary,
                   COALESCE(bool_or(school_type = '中学校'), false) AS has_junior_high
            FROM schools
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
            "#,
        );
        let row = timeout(
            TLS_QUERY_TIMEOUT,
            bind_coord(query, coord.lng(), coord.lat())
                .bind(TLS_SCHOOL_SEARCH_RADIUS_M)
                .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("tls schools_nearby query".into()))?
        .map_err(map_db_err)
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

    #[tracing::instrument(skip(self))]
    async fn find_medical_nearby(&self, coord: &Coord) -> Result<MedicalStats, DomainError> {
        // TLS_PRICE_SEARCH_RADIUS_M radius, SRID: 4326
        let query = sqlx::query_as::<_, MedicalNearbyRow>(
            r#"
            SELECT COUNT(*) FILTER (WHERE facility_type = '病院') AS hospital_count,
                   COUNT(*) FILTER (WHERE facility_type != '病院') AS clinic_count,
                   COALESCE(SUM(beds), 0)::int8 AS total_beds
            FROM medical_facilities
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
            "#,
        );
        let row = timeout(
            TLS_QUERY_TIMEOUT,
            bind_coord(query, coord.lng(), coord.lat())
                .bind(TLS_PRICE_SEARCH_RADIUS_M)
                .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("tls medical_nearby query".into()))?
        .map_err(map_db_err)
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

    #[tracing::instrument(skip(self))]
    async fn find_zoning_far(&self, coord: &Coord) -> Result<Option<f64>, DomainError> {
        // Find the zoning polygon that contains the point; return its floor_area_ratio.
        let query = sqlx::query_as::<_, OptionalF64Row>(
            r#"
            SELECT floor_area_ratio::double precision AS value
            FROM zoning
            WHERE ST_Contains(geom, ST_SetSRID(ST_MakePoint($1, $2), 4326))
            LIMIT 1
            "#,
        );
        let row = timeout(
            TLS_QUERY_TIMEOUT,
            bind_coord(query, coord.lng(), coord.lat()).fetch_optional(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("tls zoning_far query".into()))?
        .map_err(map_db_err)
        .inspect(|r| tracing::debug!(found = r.is_some(), "tls zoning_far fetched"))?;

        Ok(row.and_then(|r| r.value))
    }

    #[tracing::instrument(skip(self))]
    async fn calc_price_z_score(&self, coord: &Coord) -> Result<ZScoreResult, DomainError> {
        // Uses the denormalized zone_type column on land_prices to avoid the slow
        // ST_Contains join against the zoning table that was causing 503 errors.
        let query = sqlx::query_as::<_, ZScoreRow>(
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
        );
        let row = timeout(
            TLS_QUERY_TIMEOUT,
            bind_coord(query, coord.lng(), coord.lat())
                .bind(TLS_PRICE_SEARCH_RADIUS_M)
                .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("tls price_z_score query".into()))?
        .map_err(map_db_err)
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

    #[tracing::instrument(skip(self))]
    async fn count_recent_transactions(&self, coord: &Coord) -> Result<i64, DomainError> {
        // Count land_prices within TLS_TRANSACTION_SEARCH_RADIUS_M where year >= (max_year - 1).
        // This captures the latest full year and prior year for recency assessment.
        let query = sqlx::query_as::<_, CountRow>(
            r#"
            SELECT COUNT(*) AS count
            FROM land_prices
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $3)
              AND survey_year >= (SELECT MAX(survey_year) - 1 FROM land_prices)
            "#,
        );
        let row = timeout(
            TLS_QUERY_TIMEOUT,
            bind_coord(query, coord.lng(), coord.lat())
                .bind(TLS_TRANSACTION_SEARCH_RADIUS_M)
                .fetch_one(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("tls recent_transactions query".into()))?
        .map_err(map_db_err)
        .inspect(|r| tracing::debug!(count = r.count, "tls recent_transactions fetched"))?;

        Ok(row.count)
    }
}

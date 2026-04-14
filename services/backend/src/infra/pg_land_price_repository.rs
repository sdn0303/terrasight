//! PostgreSQL + PostGIS implementation of [`LandPriceRepository`].
//!
//! Implements [`LandPriceRepository`](crate::domain::repository::LandPriceRepository)
//! for three distinct access patterns:
//!
//! 1. **Single-year spatial query** — `find_by_year_and_bbox`: filters by
//!    `survey_year = $5` then `ST_Intersects` with `ST_MakeEnvelope`.
//! 2. **Multi-year range query** — `find_all_years_by_bbox`: `survey_year BETWEEN $5 AND $6`
//!    for the time machine animation endpoint. The feature limit is scaled by the
//!    year count so each year gets roughly the same budget.
//! 3. **Opportunity fetch** — `find_for_opportunities`: `INNER JOIN zoning` via
//!    `ST_Within` so that `building_coverage_ratio` and `floor_area_ratio` are
//!    always populated. Dynamic filters (`price_range`, `zones`) are appended
//!    with [`sqlx::QueryBuilder`] to avoid string concatenation.
//!
//! All queries use `ST_MakeEnvelope($1, $2, $3, $4, 4326)` (SRID 4326, WGS84).
//! Timeouts are enforced via [`run_query`](crate::infra::query_helpers::run_query).

use std::time::Duration;

use async_trait::async_trait;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use terrasight_geo::coord::GeoBBox;
use terrasight_geo::spatial::{LayerKind, bbox_area_deg2, compute_feature_limit};
use terrasight_server::db::spatial::bind_bbox;

use crate::domain::constants::OPPORTUNITY_QUERY_TIMEOUT_SECS;
use crate::domain::entity::{
    Address, BuildingCoverageRatio, FloorAreaRatio, GeoFeature, LayerResult, OpportunityRecord,
    PricePerSqm, ZoneCode,
};
use crate::domain::error::DomainError;
use crate::domain::repository::LandPriceRepository;
use crate::domain::value_object::{BBox, Coord, PrefCode, Year, ZoomLevel};
use crate::infra::query_helpers::{apply_limit, run_query};
use crate::infra::row_types::LandPriceFeatureRow;

/// Maximum time to wait for the land price query before returning an error.
const LAND_PRICE_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/// Longer timeout for multi-year queries which scan more rows.
const LAND_PRICE_ALL_YEARS_TIMEOUT: Duration = Duration::from_secs(10);

/// Raw row for the `/api/v1/opportunities` query. Joined with the `zoning`
/// table to populate `building_coverage_ratio` and `floor_area_ratio`.
#[derive(Debug, FromRow)]
struct OpportunityRow {
    id: i64,
    price_per_sqm: i32,
    address: String,
    zone_type: String,
    building_coverage_ratio: i32,
    floor_area_ratio: i32,
    lng: f64,
    lat: f64,
    survey_year: i16,
}

impl TryFrom<OpportunityRow> for OpportunityRecord {
    type Error = DomainError;

    fn try_from(r: OpportunityRow) -> Result<Self, DomainError> {
        Ok(Self {
            id: r.id,
            coord: Coord::new(r.lat, r.lng)?,
            address: Address::parse(&r.address)?,
            zone: ZoneCode::parse(&r.zone_type)?,
            building_coverage_ratio: BuildingCoverageRatio::new(r.building_coverage_ratio)?,
            floor_area_ratio: FloorAreaRatio::new(r.floor_area_ratio)?,
            price_per_sqm: PricePerSqm::new(i64::from(r.price_per_sqm))?,
            year: Year::new(i32::from(r.survey_year))?,
        })
    }
}

/// PostgreSQL + PostGIS implementation of [`LandPriceRepository`](crate::domain::repository::LandPriceRepository).
pub(crate) struct PgLandPriceRepository {
    pool: PgPool,
}

impl PgLandPriceRepository {
    /// Create a new repository backed by the given connection pool.
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LandPriceRepository for PgLandPriceRepository {
    /// Fetch land price features intersecting the given bounding box for the specified year.
    ///
    /// Uses `ST_Intersects` with `ST_MakeEnvelope` (SRID 4326) for spatial filtering.
    /// Applies a dynamic feature limit computed from `zoom` and the bbox area.
    /// Returns [`LayerResult`] with truncation metadata (N+1 pattern).
    #[tracing::instrument(skip(self))]
    async fn find_by_year_and_bbox(
        &self,
        year: Year,
        bbox: &BBox,
        zoom: ZoomLevel,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        let geo_bbox = GeoBBox::new(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let area = bbox_area_deg2(&geo_bbox);
        // ZoomLevel::get() returns u32; Web Mercator zoom is always 0–22, so as u8 is safe.
        let limit = compute_feature_limit(LayerKind::LandPrice, area, zoom.get() as u8);

        let rows = run_query(
            LAND_PRICE_QUERY_TIMEOUT,
            "land_price query",
            bind_bbox(
                sqlx::query_as::<_, LandPriceFeatureRow>(
                    r#"
                    SELECT id, price_per_sqm, address, land_use, survey_year,
                           ST_AsGeoJSON(geom)::jsonb AS geometry
                    FROM land_prices
                    WHERE survey_year = $5
                      AND ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                      AND ($6::text IS NULL OR pref_code = $6)
                    LIMIT $7
                    "#,
                ),
                &geo_bbox,
            )
            .bind(year.value())
            .bind(pref_code.map(PrefCode::as_str))
            .bind(limit + 1)
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| {
            tracing::debug!(
                row_count = rows.len(),
                year = year.value(),
                limit,
                "land_prices fetched"
            )
        })?;

        Ok(apply_limit(
            rows.into_iter().map(GeoFeature::from).collect(),
            limit,
        ))
    }

    /// Fetch land price features across a year range for time machine animation.
    ///
    /// Uses the same spatial filter as `find_by_year_and_bbox` but with a `BETWEEN`
    /// year clause. The feature limit is multiplied by the number of years in the
    /// range so that each year gets roughly the same budget.
    #[tracing::instrument(skip(self))]
    async fn find_all_years_by_bbox(
        &self,
        from_year: Year,
        to_year: Year,
        bbox: &BBox,
        zoom: ZoomLevel,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        let geo_bbox = GeoBBox::new(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let area = bbox_area_deg2(&geo_bbox);
        let year_count = i64::from((to_year.value() - from_year.value() + 1).max(1));
        // ZoomLevel::get() returns u32; Web Mercator zoom is always 0–22, so as u8 is safe.
        let base_limit = compute_feature_limit(LayerKind::LandPrice, area, zoom.get() as u8);
        let limit = base_limit.saturating_mul(year_count);

        let rows = run_query(
            LAND_PRICE_ALL_YEARS_TIMEOUT,
            "land_price all-years query",
            bind_bbox(
                sqlx::query_as::<_, LandPriceFeatureRow>(
                    r#"
                    SELECT id, price_per_sqm, address, land_use, survey_year,
                           ST_AsGeoJSON(geom)::jsonb AS geometry
                    FROM land_prices
                    WHERE survey_year BETWEEN $5 AND $6
                      AND ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                      AND ($7::text IS NULL OR pref_code = $7)
                    LIMIT $8
                    "#,
                ),
                &geo_bbox,
            )
            .bind(from_year.value())
            .bind(to_year.value())
            .bind(pref_code.map(PrefCode::as_str))
            .bind(limit + 1)
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| {
            tracing::debug!(
                row_count = rows.len(),
                from_year = from_year.value(),
                to_year = to_year.value(),
                limit,
                "land_prices all-years fetched"
            )
        })?;

        Ok(apply_limit(
            rows.into_iter().map(GeoFeature::from).collect(),
            limit,
        ))
    }

    /// Fetch enriched opportunity records via a spatial join with the
    /// `zoning` table. Uses [`QueryBuilder`] so optional `price_range` and
    /// `zones` filters are appended without string concatenation.
    ///
    /// Only records with a matching zoning polygon (INNER JOIN) are
    /// returned, ensuring `building_coverage_ratio` and `floor_area_ratio`
    /// are always populated. Sorted by `price_per_sqm DESC` for consistent
    /// pagination.
    #[tracing::instrument(skip(self))]
    async fn find_for_opportunities(
        &self,
        bbox: &BBox,
        limit: u32,
        price_range: Option<(PricePerSqm, PricePerSqm)>,
        zones: &[ZoneCode],
        pref_code: Option<&PrefCode>,
    ) -> Result<Vec<OpportunityRecord>, DomainError> {
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT lp.id, lp.price_per_sqm, lp.address, lp.zone_type, \
                    z.building_coverage::int AS building_coverage_ratio, \
                    z.floor_area_ratio::int AS floor_area_ratio, \
                    ST_X(lp.geom) AS lng, ST_Y(lp.geom) AS lat, lp.survey_year \
             FROM land_prices lp \
             INNER JOIN zoning z ON ST_Within(lp.geom, z.geom) \
             WHERE lp.zone_type IS NOT NULL \
               AND ST_Intersects(lp.geom, ST_MakeEnvelope(",
        );
        builder
            .push_bind(bbox.west())
            .push(", ")
            .push_bind(bbox.south())
            .push(", ")
            .push_bind(bbox.east())
            .push(", ")
            .push_bind(bbox.north())
            .push(", 4326))");

        if let Some(pc) = pref_code {
            builder
                .push(" AND lp.pref_code = ")
                .push_bind(pc.as_str().to_string());
        }

        // `price_per_sqm` is a 32-bit `integer` column. `PricePerSqm`
        // stores an `i64` internally, but the handler boundary
        // (`handler::request::opportunities::parse_price_range`)
        // rejects values outside the `i32` range with
        // `DomainError::Validation`, so this conversion is
        // guaranteed to succeed. Downgrade to a database error if
        // the invariant is ever violated by future callers rather
        // than silently clamping.
        if let Some((lo, hi)) = price_range {
            let lo_i32 = i32::try_from(lo.value()).map_err(|_| {
                DomainError::Database(format!(
                    "internal invariant violated: price_min {} does not fit in i32",
                    lo.value()
                ))
            })?;
            let hi_i32 = i32::try_from(hi.value()).map_err(|_| {
                DomainError::Database(format!(
                    "internal invariant violated: price_max {} does not fit in i32",
                    hi.value()
                ))
            })?;
            builder
                .push(" AND lp.price_per_sqm BETWEEN ")
                .push_bind(lo_i32)
                .push(" AND ")
                .push_bind(hi_i32);
        }

        if let Some((first, rest)) = zones.split_first() {
            builder
                .push(" AND lp.zone_type IN (")
                .push_bind(first.as_str().to_string());
            for z in rest {
                builder.push(", ").push_bind(z.as_str().to_string());
            }
            builder.push(")");
        }

        builder
            .push(" ORDER BY lp.price_per_sqm DESC LIMIT ")
            .push_bind(i64::from(limit));

        run_query(
            Duration::from_secs(OPPORTUNITY_QUERY_TIMEOUT_SECS),
            "opportunities query",
            builder
                .build_query_as::<OpportunityRow>()
                .fetch_all(&self.pool),
        )
        .await?
        .into_iter()
        .map(OpportunityRecord::try_from)
        .collect::<Result<Vec<_>, _>>()
        .inspect(|records| tracing::debug!(count = records.len(), "opportunities rows mapped"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_row() -> OpportunityRow {
        OpportunityRow {
            id: 42,
            price_per_sqm: 1_500_000,
            address: "東京都新宿区西新宿1-1".to_string(),
            zone_type: "商業地域".to_string(),
            building_coverage_ratio: 80,
            floor_area_ratio: 800,
            lng: 139.693,
            lat: 35.689,
            survey_year: 2024,
        }
    }

    #[test]
    fn try_from_valid_row_succeeds() {
        let row = valid_row();
        let record = OpportunityRecord::try_from(row).expect("valid row must convert");
        assert_eq!(record.id, 42);
        assert_eq!(record.price_per_sqm.value(), 1_500_000);
        assert_eq!(record.zone.as_str(), "商業地域");
        assert_eq!(record.building_coverage_ratio.value(), 80);
        assert_eq!(record.floor_area_ratio.value(), 800);
        assert_eq!(record.year.value(), 2024);
    }

    #[test]
    fn try_from_rejects_invalid_coord() {
        let row = OpportunityRow {
            lat: 91.0, // out of range
            ..valid_row()
        };
        assert!(OpportunityRecord::try_from(row).is_err());
    }

    #[test]
    fn try_from_rejects_empty_zone() {
        let row = OpportunityRow {
            zone_type: "   ".to_string(),
            ..valid_row()
        };
        assert!(OpportunityRecord::try_from(row).is_err());
    }

    #[test]
    fn try_from_rejects_bcr_out_of_range() {
        let row = OpportunityRow {
            building_coverage_ratio: 150,
            ..valid_row()
        };
        assert!(OpportunityRecord::try_from(row).is_err());
    }

    #[test]
    fn try_from_rejects_far_out_of_range() {
        let row = OpportunityRow {
            floor_area_ratio: 9999,
            ..valid_row()
        };
        assert!(OpportunityRecord::try_from(row).is_err());
    }

    #[test]
    fn try_from_rejects_negative_price() {
        let row = OpportunityRow {
            price_per_sqm: -1,
            ..valid_row()
        };
        assert!(OpportunityRecord::try_from(row).is_err());
    }
}

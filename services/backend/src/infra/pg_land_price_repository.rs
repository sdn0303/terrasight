use std::time::Duration;

use async_trait::async_trait;
use realestate_db::spatial::bind_bbox;
use realestate_geo_math::spatial::{bbox_area_deg2, compute_feature_limit};
use serde_json::json;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder};
use tokio::time::timeout;

use super::map_db_err;
use crate::domain::constants::OPPORTUNITY_QUERY_TIMEOUT_SECS;
use crate::domain::entity::{
    Address, BuildingCoverageRatio, FloorAreaRatio, GeoFeature, GeoJsonGeometry, LayerResult,
    OpportunityRecord, PricePerSqm, ZoneCode,
};
use crate::domain::error::DomainError;
use crate::domain::repository::LandPriceRepository;
use crate::domain::value_object::{
    BBox, Coord, OpportunityLimit, OpportunityOffset, Year, ZoomLevel,
};

/// Maximum time to wait for the land price query before returning an error.
const LAND_PRICE_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

/// Longer timeout for multi-year queries which scan more rows.
const LAND_PRICE_ALL_YEARS_TIMEOUT: Duration = Duration::from_secs(10);

/// Raw row returned by land-price spatial queries.
#[derive(Debug, FromRow)]
struct LandPriceFeatureRow {
    id: i64,
    price_per_sqm: i32,
    address: String,
    land_use: Option<String>,
    year: i32,
    geometry: serde_json::Value,
}

impl From<LandPriceFeatureRow> for GeoFeature {
    fn from(row: LandPriceFeatureRow) -> Self {
        to_geo_feature(
            row.geometry,
            json!({
                "id": row.id,
                "price_per_sqm": row.price_per_sqm,
                "address": row.address,
                "land_use": row.land_use,
                "year": row.year,
            }),
        )
    }
}

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
    year: i32,
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
            year: Year::new(r.year)?,
        })
    }
}

/// PostgreSQL + PostGIS implementation of [`LandPriceRepository`].
pub struct PgLandPriceRepository {
    pool: PgPool,
}

impl PgLandPriceRepository {
    pub fn new(pool: PgPool) -> Self {
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
    ) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("landprice", area, zoom.get());

        let query = sqlx::query_as::<_, LandPriceFeatureRow>(
            r#"
            SELECT id, price_per_sqm, address, land_use, year,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM land_prices
            WHERE year = $5
              AND ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $6
            "#,
        );
        let rows = timeout(
            LAND_PRICE_QUERY_TIMEOUT,
            bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
                .bind(year.value())
                .bind(limit + 1)
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("land_price query".into()))?
        .map_err(map_db_err)
        .inspect(|rows| {
            tracing::debug!(
                row_count = rows.len(),
                year = year.value(),
                limit,
                "land_prices fetched"
            )
        })?;

        let mut features: Vec<GeoFeature> = rows.into_iter().map(GeoFeature::from).collect();

        let truncated = features.len() > limit as usize;
        if truncated {
            features.truncate(limit as usize);
        }

        Ok(LayerResult {
            features,
            truncated,
            limit,
        })
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
    ) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let year_count = i64::from((to_year.value() - from_year.value() + 1).max(1));
        let base_limit = compute_feature_limit("landprice", area, zoom.get());
        let limit = base_limit.saturating_mul(year_count);

        let query = sqlx::query_as::<_, LandPriceFeatureRow>(
            r#"
            SELECT id, price_per_sqm, address, land_use, year,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM land_prices
            WHERE year BETWEEN $5 AND $6
              AND ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $7
            "#,
        );
        let rows = timeout(
            LAND_PRICE_ALL_YEARS_TIMEOUT,
            bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
                .bind(from_year.value())
                .bind(to_year.value())
                .bind(limit + 1)
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("land_price all-years query".into()))?
        .map_err(map_db_err)
        .inspect(|rows| {
            tracing::debug!(
                row_count = rows.len(),
                from_year = from_year.value(),
                to_year = to_year.value(),
                limit,
                "land_prices all-years fetched"
            )
        })?;

        let mut features: Vec<GeoFeature> = rows.into_iter().map(GeoFeature::from).collect();

        let truncated = features.len() > limit as usize;
        if truncated {
            features.truncate(limit as usize);
        }

        Ok(LayerResult {
            features,
            truncated,
            limit,
        })
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
        limit: OpportunityLimit,
        offset: OpportunityOffset,
        price_range: Option<(PricePerSqm, PricePerSqm)>,
        zones: &[ZoneCode],
    ) -> Result<Vec<OpportunityRecord>, DomainError> {
        let mut builder = QueryBuilder::<Postgres>::new(
            "SELECT lp.id, lp.price_per_sqm, lp.address, lp.zone_type, \
                    z.building_coverage::int AS building_coverage_ratio, \
                    z.floor_area_ratio::int AS floor_area_ratio, \
                    ST_X(lp.geom) AS lng, ST_Y(lp.geom) AS lat, lp.year \
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

        if let Some((lo, hi)) = price_range {
            builder.push(" AND lp.price_per_sqm BETWEEN ");
            builder
                .push_bind(i32::try_from(lo.value()).unwrap_or(i32::MAX))
                .push(" AND ")
                .push_bind(i32::try_from(hi.value()).unwrap_or(i32::MAX));
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
            .push_bind(i64::from(limit.get()))
            .push(" OFFSET ")
            .push_bind(i64::from(offset.get()));

        timeout(
            Duration::from_secs(OPPORTUNITY_QUERY_TIMEOUT_SECS),
            builder
                .build_query_as::<OpportunityRow>()
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("opportunities query".into()))?
        .map_err(map_db_err)?
        .into_iter()
        .map(OpportunityRecord::try_from)
        .collect::<Result<Vec<_>, _>>()
        .inspect(|records| tracing::debug!(count = records.len(), "opportunities rows mapped"))
    }
}

/// Parse PostGIS `ST_AsGeoJSON` output into a domain [`GeoFeature`].
fn to_geo_feature(geojson: serde_json::Value, properties: serde_json::Value) -> GeoFeature {
    let raw = realestate_db::geo::to_raw_geo_feature(geojson, properties);
    GeoFeature {
        geometry: GeoJsonGeometry {
            r#type: raw.geo_type,
            coordinates: raw.coordinates,
        },
        properties: raw.properties,
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
            year: 2024,
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

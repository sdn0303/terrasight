use std::time::Duration;

use async_trait::async_trait;
use realestate_db::spatial::bind_bbox;
use realestate_geo_math::spatial::{bbox_area_deg2, compute_feature_limit};
use serde_json::json;
use sqlx::{FromRow, PgPool};
use tokio::time::timeout;

use super::map_db_err;
use crate::domain::entity::{GeoFeature, GeoJsonGeometry, LayerResult};
use crate::domain::error::DomainError;
use crate::domain::repository::LandPriceRepository;
use crate::domain::value_object::{BBox, Year, ZoomLevel};

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
        .map_err(|_| DomainError::Database("land_price query timed out".into()))?
        .map_err(map_db_err)?;

        tracing::debug!(
            row_count = rows.len(),
            year = year.value(),
            limit,
            "land_prices fetched"
        );

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
        .map_err(|_| DomainError::Database("land_price all-years query timed out".into()))?
        .map_err(map_db_err)?;

        tracing::debug!(
            row_count = rows.len(),
            from_year = from_year.value(),
            to_year = to_year.value(),
            limit,
            "land_prices all-years fetched"
        );

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

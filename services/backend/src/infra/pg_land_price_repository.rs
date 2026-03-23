use std::time::Duration;

use async_trait::async_trait;
use realestate_db::spatial::bind_bbox;
use serde_json::json;
use sqlx::PgPool;
use tokio::time::timeout;

use super::map_db_err;
use crate::domain::entity::{GeoFeature, GeoJsonGeometry};
use crate::domain::error::DomainError;
use crate::domain::repository::LandPriceRepository;
use crate::domain::value_object::{BBox, Year};

/// Maximum time to wait for the land price query before returning an error.
const LAND_PRICE_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

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
    #[tracing::instrument(skip(self))]
    async fn find_by_year_and_bbox(
        &self,
        year: &Year,
        bbox: &BBox,
    ) -> Result<Vec<GeoFeature>, DomainError> {
        let query = sqlx::query_as::<_, (i64, i32, String, Option<String>, i32, serde_json::Value)>(
            r#"
            SELECT id, price_per_sqm, address, land_use, year,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM land_prices
            WHERE year = $5
              AND ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            "#,
        );
        let rows = timeout(
            LAND_PRICE_QUERY_TIMEOUT,
            bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
                .bind(year.value())
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Database("land_price query timed out".into()))?
        .map_err(map_db_err)?;

        tracing::debug!(
            row_count = rows.len(),
            year = year.value(),
            "land_prices fetched"
        );

        Ok(rows
            .into_iter()
            .map(|(id, price, address, land_use, row_year, geom)| {
                to_geo_feature(
                    geom,
                    json!({
                        "id": id,
                        "price_per_sqm": price,
                        "address": address,
                        "land_use": land_use,
                        "year": row_year,
                    }),
                )
            })
            .collect())
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

use async_trait::async_trait;
use realestate_db::spatial::bind_bbox;
use serde_json::json;
use sqlx::PgPool;

use super::map_db_err;
use crate::domain::entity::{GeoFeature, GeoJsonGeometry};
use crate::domain::error::DomainError;
use crate::domain::repository::AreaRepository;
use crate::domain::value_object::BBox;

pub struct PgAreaRepository {
    pool: PgPool,
}

impl PgAreaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AreaRepository for PgAreaRepository {
    #[tracing::instrument(skip(self))]
    async fn find_land_prices(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        let query = sqlx::query_as::<_, (i64, i32, String, Option<String>, i32, serde_json::Value)>(
            r#"
            SELECT id, price_per_sqm, address, land_use, year,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM land_prices
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), "land_prices fetched");

        Ok(rows
            .into_iter()
            .map(|(id, price, address, land_use, year, geom)| {
                to_geo_feature(
                    geom,
                    json!({
                        "id": id,
                        "price_per_sqm": price,
                        "address": address,
                        "land_use": land_use,
                        "year": year,
                    }),
                )
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn find_zoning(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        let query = sqlx::query_as::<
            _,
            (
                i64,
                String,
                Option<String>,
                Option<f32>,
                Option<f32>,
                serde_json::Value,
            ),
        >(
            r#"
            SELECT id, zone_type, zone_code, floor_area_ratio, building_coverage,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM zoning
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), "zoning fetched");

        Ok(rows
            .into_iter()
            .map(|(id, zone_type, zone_code, far, bc, geom)| {
                to_geo_feature(
                    geom,
                    json!({
                        "id": id,
                        "zone_type": zone_type,
                        "zone_code": zone_code,
                        "floor_area_ratio": far,
                        "building_coverage": bc,
                    }),
                )
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn find_flood_risk(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        let query = sqlx::query_as::<_, (i64, Option<String>, Option<String>, serde_json::Value)>(
            r#"
            SELECT id, depth_rank, river_name,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM flood_risk
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), "flood_risk fetched");

        Ok(rows
            .into_iter()
            .map(|(id, depth_rank, river_name, geom)| {
                to_geo_feature(
                    geom,
                    json!({
                        "id": id,
                        "depth_rank": depth_rank,
                        "river_name": river_name,
                    }),
                )
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn find_steep_slope(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        let query = sqlx::query_as::<_, (i64, Option<String>, serde_json::Value)>(
            r#"
            SELECT id, area_name,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM steep_slope
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), "steep_slope fetched");

        Ok(rows
            .into_iter()
            .map(|(id, area_name, geom)| {
                to_geo_feature(geom, json!({ "id": id, "area_name": area_name }))
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn find_schools(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        let query = sqlx::query_as::<_, (i64, String, Option<String>, serde_json::Value)>(
            r#"
            SELECT id, name, school_type,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM schools
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), "schools fetched");

        Ok(rows
            .into_iter()
            .map(|(id, name, school_type, geom)| {
                to_geo_feature(
                    geom,
                    json!({ "id": id, "name": name, "school_type": school_type }),
                )
            })
            .collect())
    }

    #[tracing::instrument(skip(self))]
    async fn find_medical(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        let query =
            sqlx::query_as::<_, (i64, String, Option<String>, Option<i32>, serde_json::Value)>(
                r#"
            SELECT id, name, facility_type, bed_count,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM medical_facilities
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            "#,
            );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), "medical_facilities fetched");

        Ok(rows
            .into_iter()
            .map(|(id, name, facility_type, bed_count, geom)| {
                to_geo_feature(
                    geom,
                    json!({
                        "id": id,
                        "name": name,
                        "facility_type": facility_type,
                        "bed_count": bed_count,
                    }),
                )
            })
            .collect())
    }
}

/// Parse PostGIS `ST_AsGeoJSON` output into domain [`GeoFeature`].
///
/// Delegates geometry parsing to [`realestate_db::geo::to_raw_geo_feature`] and
/// then maps the domain-independent result to the application's domain type.
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

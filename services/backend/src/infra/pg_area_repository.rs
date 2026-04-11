use async_trait::async_trait;
use realestate_db::spatial::bind_bbox;
use realestate_geo_math::spatial::{bbox_area_deg2, compute_feature_limit};
use serde_json::json;
use sqlx::PgPool;

use super::map_db_err;
use crate::domain::entity::{GeoFeature, GeoJsonGeometry, LayerResult};
use crate::domain::error::DomainError;
use crate::domain::repository::LayerRepository;
use crate::domain::value_object::{BBox, LayerType};

pub struct PgAreaRepository {
    pool: PgPool,
}

impl PgAreaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// Apply the N+1 truncation pattern: fetch `limit + 1` rows, check if more
/// exist, then return at most `limit` rows along with the truncation flag.
fn apply_limit(mut rows: Vec<GeoFeature>, limit: i64) -> LayerResult {
    let truncated = rows.len() > limit as usize;
    if truncated {
        rows.truncate(limit as usize);
    }
    LayerResult {
        features: rows,
        truncated,
        limit,
    }
}

#[async_trait]
impl LayerRepository for PgAreaRepository {
    #[tracing::instrument(skip(self), fields(layer = ?layer))]
    async fn find_layer(
        &self,
        layer: LayerType,
        bbox: &BBox,
        zoom: u32,
    ) -> Result<LayerResult, DomainError> {
        match layer {
            LayerType::LandPrice => self.query_land_prices(bbox, zoom).await,
            LayerType::Zoning => self.query_zoning(bbox, zoom).await,
            LayerType::Flood => self.query_flood_risk(bbox, zoom).await,
            LayerType::SteepSlope => self.query_steep_slope(bbox, zoom).await,
            LayerType::Schools => self.query_schools(bbox, zoom).await,
            LayerType::Medical => self.query_medical(bbox, zoom).await,
        }
    }
}

impl PgAreaRepository {
    async fn query_land_prices(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("landprice", area, zoom);
        let query = sqlx::query_as::<_, (i64, i32, String, Option<String>, i32, serde_json::Value)>(
            r#"
            SELECT id, price_per_sqm, address, land_use, year,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM land_prices
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .bind(limit + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), limit, "land_prices fetched");

        let features: Vec<GeoFeature> = rows
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
            .collect();

        Ok(apply_limit(features, limit))
    }

    async fn query_zoning(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("zoning", area, zoom);
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
            LIMIT $5
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .bind(limit + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), limit, "zoning fetched");

        let features: Vec<GeoFeature> = rows
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
            .collect();

        Ok(apply_limit(features, limit))
    }

    async fn query_flood_risk(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("flood", area, zoom);
        let query = sqlx::query_as::<_, (i64, Option<i16>, Option<String>, serde_json::Value)>(
            r#"
            SELECT id, depth_rank, river_name,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM flood_risk
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .bind(limit + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), limit, "flood_risk fetched");

        let features: Vec<GeoFeature> = rows
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
            .collect();

        Ok(apply_limit(features, limit))
    }

    async fn query_steep_slope(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("steep_slope", area, zoom);
        let query = sqlx::query_as::<_, (i64, Option<String>, serde_json::Value)>(
            r#"
            SELECT id, area_name,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM steep_slope
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .bind(limit + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), limit, "steep_slope fetched");

        let features: Vec<GeoFeature> = rows
            .into_iter()
            .map(|(id, area_name, geom)| {
                to_geo_feature(geom, json!({ "id": id, "area_name": area_name }))
            })
            .collect();

        Ok(apply_limit(features, limit))
    }

    async fn query_schools(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("schools", area, zoom);
        let query = sqlx::query_as::<_, (i64, String, Option<String>, serde_json::Value)>(
            r#"
            SELECT id, name, school_type,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM schools
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
        );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .bind(limit + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), limit, "schools fetched");

        let features: Vec<GeoFeature> = rows
            .into_iter()
            .map(|(id, name, school_type, geom)| {
                to_geo_feature(
                    geom,
                    json!({ "id": id, "name": name, "school_type": school_type }),
                )
            })
            .collect();

        Ok(apply_limit(features, limit))
    }

    async fn query_medical(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("medical", area, zoom);
        let query =
            sqlx::query_as::<_, (i64, String, Option<String>, Option<i32>, serde_json::Value)>(
                r#"
            SELECT id, name, facility_type, bed_count,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM medical_facilities
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
            );
        let rows = bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
            .bind(limit + 1)
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_err)?;
        tracing::debug!(row_count = rows.len(), limit, "medical_facilities fetched");

        let features: Vec<GeoFeature> = rows
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
            .collect();

        Ok(apply_limit(features, limit))
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

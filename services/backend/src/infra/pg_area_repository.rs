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
use crate::domain::repository::LayerRepository;
use crate::domain::value_object::{BBox, LayerType, ZoomLevel};

/// Maximum time to wait for any single layer query before returning an error.
const LAYER_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, FromRow)]
struct LandPriceLayerRow {
    id: i64,
    price_per_sqm: i32,
    address: String,
    land_use: Option<String>,
    year: i32,
    geometry: serde_json::Value,
}

impl From<LandPriceLayerRow> for GeoFeature {
    fn from(row: LandPriceLayerRow) -> Self {
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

#[derive(Debug, FromRow)]
struct ZoningLayerRow {
    id: i64,
    zone_type: String,
    zone_code: Option<String>,
    floor_area_ratio: Option<f32>,
    building_coverage: Option<f32>,
    geometry: serde_json::Value,
}

impl From<ZoningLayerRow> for GeoFeature {
    fn from(row: ZoningLayerRow) -> Self {
        to_geo_feature(
            row.geometry,
            json!({
                "id": row.id,
                "zone_type": row.zone_type,
                "zone_code": row.zone_code,
                "floor_area_ratio": row.floor_area_ratio,
                "building_coverage": row.building_coverage,
            }),
        )
    }
}

#[derive(Debug, FromRow)]
struct FloodRiskRow {
    id: i64,
    depth_rank: Option<i16>,
    river_name: Option<String>,
    geometry: serde_json::Value,
}

impl From<FloodRiskRow> for GeoFeature {
    fn from(row: FloodRiskRow) -> Self {
        to_geo_feature(
            row.geometry,
            json!({
                "id": row.id,
                "depth_rank": row.depth_rank,
                "river_name": row.river_name,
            }),
        )
    }
}

#[derive(Debug, FromRow)]
struct SteepSlopeRow {
    id: i64,
    area_name: Option<String>,
    geometry: serde_json::Value,
}

impl From<SteepSlopeRow> for GeoFeature {
    fn from(row: SteepSlopeRow) -> Self {
        to_geo_feature(
            row.geometry,
            json!({ "id": row.id, "area_name": row.area_name }),
        )
    }
}

#[derive(Debug, FromRow)]
struct SchoolRow {
    id: i64,
    name: String,
    school_type: Option<String>,
    geometry: serde_json::Value,
}

impl From<SchoolRow> for GeoFeature {
    fn from(row: SchoolRow) -> Self {
        to_geo_feature(
            row.geometry,
            json!({ "id": row.id, "name": row.name, "school_type": row.school_type }),
        )
    }
}

#[derive(Debug, FromRow)]
struct MedicalFacilityRow {
    id: i64,
    name: String,
    facility_type: Option<String>,
    bed_count: Option<i32>,
    geometry: serde_json::Value,
}

impl From<MedicalFacilityRow> for GeoFeature {
    fn from(row: MedicalFacilityRow) -> Self {
        to_geo_feature(
            row.geometry,
            json!({
                "id": row.id,
                "name": row.name,
                "facility_type": row.facility_type,
                "bed_count": row.bed_count,
            }),
        )
    }
}

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
        zoom: ZoomLevel,
    ) -> Result<LayerResult, DomainError> {
        let z = zoom.get();
        match layer {
            LayerType::LandPrice => self.query_land_prices(bbox, z).await,
            LayerType::Zoning => self.query_zoning(bbox, z).await,
            LayerType::Flood => self.query_flood_risk(bbox, z).await,
            LayerType::SteepSlope => self.query_steep_slope(bbox, z).await,
            LayerType::Schools => self.query_schools(bbox, z).await,
            LayerType::Medical => self.query_medical(bbox, z).await,
        }
    }
}

impl PgAreaRepository {
    async fn query_land_prices(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("landprice", area, zoom);
        let query = sqlx::query_as::<_, LandPriceLayerRow>(
            r#"
            SELECT id, price_per_sqm, address, land_use, year,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM land_prices
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
        );
        let rows = timeout(
            LAYER_QUERY_TIMEOUT,
            bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
                .bind(limit + 1)
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("land_prices layer query".into()))?
        .map_err(map_db_err)
        .inspect(|rows| tracing::debug!(row_count = rows.len(), limit, "land_prices fetched"))?;

        let features: Vec<GeoFeature> = rows.into_iter().map(GeoFeature::from).collect();

        Ok(apply_limit(features, limit))
    }

    async fn query_zoning(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("zoning", area, zoom);
        let query = sqlx::query_as::<_, ZoningLayerRow>(
            r#"
            SELECT id, zone_type, zone_code, floor_area_ratio, building_coverage,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM zoning
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
        );
        let rows = timeout(
            LAYER_QUERY_TIMEOUT,
            bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
                .bind(limit + 1)
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("zoning layer query".into()))?
        .map_err(map_db_err)
        .inspect(|rows| tracing::debug!(row_count = rows.len(), limit, "zoning fetched"))?;

        let features: Vec<GeoFeature> = rows.into_iter().map(GeoFeature::from).collect();

        Ok(apply_limit(features, limit))
    }

    async fn query_flood_risk(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("flood", area, zoom);
        let query = sqlx::query_as::<_, FloodRiskRow>(
            r#"
            SELECT id, depth_rank, river_name,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM flood_risk
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
        );
        let rows = timeout(
            LAYER_QUERY_TIMEOUT,
            bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
                .bind(limit + 1)
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("flood_risk layer query".into()))?
        .map_err(map_db_err)
        .inspect(|rows| tracing::debug!(row_count = rows.len(), limit, "flood_risk fetched"))?;

        let features: Vec<GeoFeature> = rows.into_iter().map(GeoFeature::from).collect();

        Ok(apply_limit(features, limit))
    }

    async fn query_steep_slope(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("steep_slope", area, zoom);
        let query = sqlx::query_as::<_, SteepSlopeRow>(
            r#"
            SELECT id, area_name,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM steep_slope
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
        );
        let rows = timeout(
            LAYER_QUERY_TIMEOUT,
            bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
                .bind(limit + 1)
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("steep_slope layer query".into()))?
        .map_err(map_db_err)
        .inspect(|rows| tracing::debug!(row_count = rows.len(), limit, "steep_slope fetched"))?;

        let features: Vec<GeoFeature> = rows.into_iter().map(GeoFeature::from).collect();

        Ok(apply_limit(features, limit))
    }

    async fn query_schools(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("schools", area, zoom);
        let query = sqlx::query_as::<_, SchoolRow>(
            r#"
            SELECT id, name, school_type,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM schools
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
        );
        let rows = timeout(
            LAYER_QUERY_TIMEOUT,
            bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
                .bind(limit + 1)
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("schools layer query".into()))?
        .map_err(map_db_err)
        .inspect(|rows| tracing::debug!(row_count = rows.len(), limit, "schools fetched"))?;

        let features: Vec<GeoFeature> = rows.into_iter().map(GeoFeature::from).collect();

        Ok(apply_limit(features, limit))
    }

    async fn query_medical(&self, bbox: &BBox, zoom: u32) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit("medical", area, zoom);
        let query = sqlx::query_as::<_, MedicalFacilityRow>(
            r#"
            SELECT id, name, facility_type, bed_count,
                   ST_AsGeoJSON(geom)::jsonb AS geometry
            FROM medical_facilities
            WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
            LIMIT $5
            "#,
        );
        let rows = timeout(
            LAYER_QUERY_TIMEOUT,
            bind_bbox(query, bbox.west(), bbox.south(), bbox.east(), bbox.north())
                .bind(limit + 1)
                .fetch_all(&self.pool),
        )
        .await
        .map_err(|_| DomainError::Timeout("medical_facilities layer query".into()))?
        .map_err(map_db_err)
        .inspect(|rows| {
            tracing::debug!(row_count = rows.len(), limit, "medical_facilities fetched")
        })?;

        let features: Vec<GeoFeature> = rows.into_iter().map(GeoFeature::from).collect();

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

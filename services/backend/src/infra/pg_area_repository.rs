//! PostgreSQL + PostGIS implementation of [`LayerRepository`].
//!
//! Implements [`LayerRepository`](crate::domain::repository::LayerRepository)
//! by dispatching over [`LayerType`] to six private query methods, each of
//! which queries a distinct PostGIS table.
//!
//! ## SQL strategy
//!
//! All spatial queries use `ST_Intersects(geom, ST_MakeEnvelope($1,$2,$3,$4, 4326))`
//! for bounding-box filtering. Feature counts are bounded by
//! [`compute_feature_limit`](terrasight_geo::spatial::compute_feature_limit) —
//! the repository requests `limit + 1` rows and [`apply_limit`](crate::infra::query_helpers::apply_limit)
//! uses the N+1 pattern to detect truncation without a separate `COUNT(*)` query.
//!
//! All queries are wrapped with [`run_query`](crate::infra::query_helpers::run_query)
//! which enforces [`LAYER_QUERY_TIMEOUT`] via `tokio::time::timeout`.

use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;
use sqlx::{FromRow, PgPool};
use terrasight_geo::spatial::{LayerKind, bbox_area_deg2, compute_feature_limit};
use terrasight_server::db::spatial::bind_bbox;

use crate::domain::entity::{GeoFeature, LayerResult};
use crate::domain::error::DomainError;
use crate::domain::repository::LayerRepository;
use crate::domain::value_object::{BBox, LayerType, PrefCode, ZoomLevel};
use crate::infra::geo_convert::to_geo_feature;
use crate::infra::query_helpers::{apply_limit, run_query};
use crate::infra::row_types::LandPriceFeatureRow;

/// Maximum time to wait for any single layer query before returning an error.
const LAYER_QUERY_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, FromRow)]
struct ZoningLayerRow {
    id: i64,
    zone_type: String,
    zone_code: String,
    floor_area_ratio: Option<f64>,
    building_coverage: Option<f64>,
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
    depth_rank: Option<String>,
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
    school_name: String,
    school_type: String,
    geometry: serde_json::Value,
}

impl From<SchoolRow> for GeoFeature {
    fn from(row: SchoolRow) -> Self {
        to_geo_feature(
            row.geometry,
            json!({ "id": row.id, "school_name": row.school_name, "school_type": row.school_type }),
        )
    }
}

#[derive(Debug, FromRow)]
struct MedicalFacilityRow {
    id: i64,
    facility_name: String,
    facility_type: String,
    beds: Option<i32>,
    geometry: serde_json::Value,
}

impl From<MedicalFacilityRow> for GeoFeature {
    fn from(row: MedicalFacilityRow) -> Self {
        to_geo_feature(
            row.geometry,
            json!({
                "id": row.id,
                "facility_name": row.facility_name,
                "facility_type": row.facility_type,
                "beds": row.beds,
            }),
        )
    }
}

/// PostgreSQL + PostGIS implementation of [`LayerRepository`].
pub(crate) struct PgAreaRepository {
    pool: PgPool,
}

impl PgAreaRepository {
    /// Create a new repository backed by the given connection pool.
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LayerRepository for PgAreaRepository {
    /// Dispatch to the per-layer query method matching `layer`.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Timeout`] if the query exceeds
    /// [`LAYER_QUERY_TIMEOUT`], or [`DomainError::Database`] on a
    /// PostgreSQL error.
    #[tracing::instrument(skip(self), fields(layer = ?layer))]
    async fn find_layer(
        &self,
        layer: LayerType,
        bbox: &BBox,
        zoom: ZoomLevel,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        let z = zoom.get();
        match layer {
            LayerType::LandPrice => self.query_land_prices(bbox, z, pref_code).await,
            LayerType::Zoning => self.query_zoning(bbox, z, pref_code).await,
            LayerType::Flood => self.query_flood_risk(bbox, z, pref_code).await,
            LayerType::SteepSlope => self.query_steep_slope(bbox, z, pref_code).await,
            LayerType::Schools => self.query_schools(bbox, z, pref_code).await,
            LayerType::Medical => self.query_medical(bbox, z, pref_code).await,
        }
    }
}

impl PgAreaRepository {
    async fn query_land_prices(
        &self,
        bbox: &BBox,
        zoom: u32,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit(LayerKind::LandPrice, area, zoom);
        let rows = run_query(
            LAYER_QUERY_TIMEOUT,
            "land_prices layer query",
            bind_bbox(
                sqlx::query_as::<_, LandPriceFeatureRow>(
                    r#"
                    SELECT id, price_per_sqm, address, land_use, survey_year,
                           ST_AsGeoJSON(geom)::jsonb AS geometry
                    FROM land_prices
                    WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                      AND ($5::text IS NULL OR pref_code = $5)
                    LIMIT $6
                    "#,
                ),
                bbox.west(),
                bbox.south(),
                bbox.east(),
                bbox.north(),
            )
            .bind(pref_code.map(PrefCode::as_str))
            .bind(limit + 1)
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| tracing::debug!(row_count = rows.len(), limit, "land_prices fetched"))?;

        Ok(apply_limit(
            rows.into_iter().map(GeoFeature::from).collect(),
            limit,
        ))
    }

    async fn query_zoning(
        &self,
        bbox: &BBox,
        zoom: u32,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit(LayerKind::Zoning, area, zoom);
        let rows = run_query(
            LAYER_QUERY_TIMEOUT,
            "zoning layer query",
            bind_bbox(
                sqlx::query_as::<_, ZoningLayerRow>(
                    r#"
                    SELECT id, zone_type, zone_code, floor_area_ratio, building_coverage,
                           ST_AsGeoJSON(geom)::jsonb AS geometry
                    FROM zoning
                    WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                      AND ($5::text IS NULL OR pref_code = $5)
                    LIMIT $6
                    "#,
                ),
                bbox.west(),
                bbox.south(),
                bbox.east(),
                bbox.north(),
            )
            .bind(pref_code.map(PrefCode::as_str))
            .bind(limit + 1)
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| tracing::debug!(row_count = rows.len(), limit, "zoning fetched"))?;

        Ok(apply_limit(
            rows.into_iter().map(GeoFeature::from).collect(),
            limit,
        ))
    }

    async fn query_flood_risk(
        &self,
        bbox: &BBox,
        zoom: u32,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit(LayerKind::Flood, area, zoom);
        let rows = run_query(
            LAYER_QUERY_TIMEOUT,
            "flood_risk layer query",
            bind_bbox(
                sqlx::query_as::<_, FloodRiskRow>(
                    r#"
                    SELECT id, depth_rank, river_name,
                           ST_AsGeoJSON(geom)::jsonb AS geometry
                    FROM flood_risk
                    WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                      AND ($5::text IS NULL OR pref_code = $5)
                    LIMIT $6
                    "#,
                ),
                bbox.west(),
                bbox.south(),
                bbox.east(),
                bbox.north(),
            )
            .bind(pref_code.map(PrefCode::as_str))
            .bind(limit + 1)
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| tracing::debug!(row_count = rows.len(), limit, "flood_risk fetched"))?;

        Ok(apply_limit(
            rows.into_iter().map(GeoFeature::from).collect(),
            limit,
        ))
    }

    async fn query_steep_slope(
        &self,
        bbox: &BBox,
        zoom: u32,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit(LayerKind::SteepSlope, area, zoom);
        let rows = run_query(
            LAYER_QUERY_TIMEOUT,
            "steep_slope layer query",
            bind_bbox(
                sqlx::query_as::<_, SteepSlopeRow>(
                    r#"
                    SELECT id, area_name,
                           ST_AsGeoJSON(geom)::jsonb AS geometry
                    FROM steep_slope
                    WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                      AND ($5::text IS NULL OR pref_code = $5)
                    LIMIT $6
                    "#,
                ),
                bbox.west(),
                bbox.south(),
                bbox.east(),
                bbox.north(),
            )
            .bind(pref_code.map(PrefCode::as_str))
            .bind(limit + 1)
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| tracing::debug!(row_count = rows.len(), limit, "steep_slope fetched"))?;

        Ok(apply_limit(
            rows.into_iter().map(GeoFeature::from).collect(),
            limit,
        ))
    }

    async fn query_schools(
        &self,
        bbox: &BBox,
        zoom: u32,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit(LayerKind::Schools, area, zoom);
        let rows = run_query(
            LAYER_QUERY_TIMEOUT,
            "schools layer query",
            bind_bbox(
                sqlx::query_as::<_, SchoolRow>(
                    r#"
                    SELECT id, school_name, school_type,
                           ST_AsGeoJSON(geom)::jsonb AS geometry
                    FROM schools
                    WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                      AND ($5::text IS NULL OR pref_code = $5)
                    LIMIT $6
                    "#,
                ),
                bbox.west(),
                bbox.south(),
                bbox.east(),
                bbox.north(),
            )
            .bind(pref_code.map(PrefCode::as_str))
            .bind(limit + 1)
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| tracing::debug!(row_count = rows.len(), limit, "schools fetched"))?;

        Ok(apply_limit(
            rows.into_iter().map(GeoFeature::from).collect(),
            limit,
        ))
    }

    async fn query_medical(
        &self,
        bbox: &BBox,
        zoom: u32,
        pref_code: Option<&PrefCode>,
    ) -> Result<LayerResult, DomainError> {
        let area = bbox_area_deg2(bbox.south(), bbox.west(), bbox.north(), bbox.east());
        let limit = compute_feature_limit(LayerKind::Medical, area, zoom);
        let rows = run_query(
            LAYER_QUERY_TIMEOUT,
            "medical_facilities layer query",
            bind_bbox(
                sqlx::query_as::<_, MedicalFacilityRow>(
                    r#"
                    SELECT id, facility_name, facility_type, beds,
                           ST_AsGeoJSON(geom)::jsonb AS geometry
                    FROM medical_facilities
                    WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
                      AND ($5::text IS NULL OR pref_code = $5)
                    LIMIT $6
                    "#,
                ),
                bbox.west(),
                bbox.south(),
                bbox.east(),
                bbox.north(),
            )
            .bind(pref_code.map(PrefCode::as_str))
            .bind(limit + 1)
            .fetch_all(&self.pool),
        )
        .await
        .inspect(|rows| {
            tracing::debug!(row_count = rows.len(), limit, "medical_facilities fetched")
        })?;

        Ok(apply_limit(
            rows.into_iter().map(GeoFeature::from).collect(),
            limit,
        ))
    }
}

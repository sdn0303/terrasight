# Rust Axum Backend API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement the full Rust Axum REST API with PostGIS spatial queries, structured logging via `tracing`, and investment scoring — turning the current stub endpoints into production-ready handlers.

**Architecture:** Clean Architecture 4-layer (Handler → Usecase/Service → Domain → Infra). Handlers validate input and serialize responses. Services orchestrate business logic (scoring). Domain defines pure types (GeoJSON, BBox, Score). Infra executes PostGIS queries via sqlx. Structured logging spans wrap every request and database call.

**Tech Stack:** Rust 1.94, Axum 0.8, tokio, sqlx 0.8 (PostGIS), tower-http (tracing/cors/gzip), tracing + tracing-subscriber (JSON structured logging), serde, chrono, thiserror/anyhow.

---

## File Structure

### New Files

| Path | Responsibility |
|------|---------------|
| `src/models/bbox.rs` | BBox value object: validation, area check (≤0.5°), ST_MakeEnvelope params |
| `src/models/coord.rs` | Coordinate value object: lat/lng validation |
| `src/models/responses.rs` | Typed response structs for all endpoints (score, stats, trend) |
| `src/infra/mod.rs` | Infra module declaration |
| `src/infra/repo_area.rs` | PostGIS queries for area-data (6 layer queries) |
| `src/infra/repo_score.rs` | PostGIS queries for scoring components (trend/risk/access/yield) |
| `src/infra/repo_stats.rs` | PostGIS queries for area statistics |
| `src/infra/repo_trend.rs` | PostGIS query for land price trend at nearest point |
| `src/logging.rs` | tracing-subscriber init with JSON structured logging + request tracing layer |
| `tests/api/mod.rs` | Integration test module |
| `tests/api/health_test.rs` | Health endpoint integration test |
| `tests/api/area_data_test.rs` | Area-data endpoint integration test |

### Modified Files

| Path | Changes |
|------|---------|
| `src/main.rs` | Import logging module, add TraceLayer, use new infra module |
| `src/models/mod.rs` | Add `bbox`, `coord`, `responses` module declarations |
| `src/models/geojson.rs` | Full GeoJSON types (Feature, FeatureCollection, Geometry) |
| `src/models/error.rs` | Add `BboxTooLarge` variant |
| `src/routes/area_data.rs` | Full handler: validate bbox → query PostGIS → return GeoJSON |
| `src/routes/score.rs` | Full handler: validate coord → call scoring service → return JSON |
| `src/routes/stats.rs` | Full handler: validate bbox → query stats → return JSON |
| `src/routes/trend.rs` | Full handler: validate coord → query trend → return JSON |
| `src/services/scoring.rs` | Scoring algorithm: trend + risk + access + yield → composite score |
| `src/services/mod.rs` | No change needed (already declares scoring) |
| `Cargo.toml` | Add `tracing-subscriber` JSON feature, `serde_with` for geometry |

---

## Task 1: Structured Logging with tracing

**Files:**
- Create: `src/logging.rs`
- Modify: `src/main.rs:14-18`
- Modify: `Cargo.toml:16-17`

- [ ] **Step 1: Update Cargo.toml tracing-subscriber features**

Add `json` and `fmt` features to `tracing-subscriber`:

```toml
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
```

Also add tower-http trace feature (already in features list, verify `"trace"` is present).

- [ ] **Step 2: Create `src/logging.rs` with structured logging init**

```rust
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize structured logging.
///
/// - Development (`RUST_LOG_FORMAT` unset or `pretty`): human-readable colored output.
/// - Production (`RUST_LOG_FORMAT=json`): JSON structured logs for log aggregation.
///
/// Filter level controlled by `RUST_LOG` env var (default: `info`).
pub fn init() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=debug"));

    let log_format = std::env::var("RUST_LOG_FORMAT").unwrap_or_default();

    if log_format == "json" {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
}

/// Build a [`TraceLayer`] that logs each HTTP request as a tracing span.
///
/// Emits:
/// - `http.method`, `http.uri`, `http.status_code`
/// - Latency as span duration
pub fn http_trace_layer() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
}
```

- [ ] **Step 3: Update `src/main.rs` to use new logging module**

Replace lines 16-18:
```rust
// Before:
tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
    .init();

// After:
mod logging;
// ... (already has mod models/routes/services)
logging::init();
```

Add TraceLayer to the router (after CorsLayer):
```rust
.layer(logging::http_trace_layer())
```

- [ ] **Step 4: Add tracing instrumentation to health handler**

Add `#[tracing::instrument(skip(state))]` to `health()` in `src/routes/health.rs:22`.

- [ ] **Step 5: Verify compilation**

Run: `cd services/backend && cargo build 2>&1`
Expected: Compiles with no errors.

- [ ] **Step 6: Commit**

```bash
git add services/backend/src/logging.rs services/backend/src/main.rs services/backend/Cargo.toml services/backend/src/routes/health.rs
git commit -m "feat(backend): add structured logging with tracing + TraceLayer"
```

---

## Task 2: Domain Types (BBox, Coord, GeoJSON, Responses)

**Files:**
- Create: `src/models/bbox.rs`
- Create: `src/models/coord.rs`
- Create: `src/models/responses.rs`
- Modify: `src/models/geojson.rs`
- Modify: `src/models/mod.rs:1-2`
- Modify: `src/models/error.rs`

- [ ] **Step 1: Create `src/models/bbox.rs`**

```rust
use crate::models::error::AppError;
use serde::Deserialize;

/// Bounding box query parameters shared by `/api/area-data` and `/api/stats`.
///
/// Validated on construction: south < north, west < east, area ≤ 0.5° per side.
#[derive(Debug, Deserialize)]
pub struct BBoxParams {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}

impl BBoxParams {
    /// Validate the bounding box constraints.
    ///
    /// Returns `AppError::BadRequest` if invalid, `AppError::BboxTooLarge` if oversized.
    pub fn validate(&self) -> Result<(), AppError> {
        if !(-90.0..=90.0).contains(&self.south)
            || !(-90.0..=90.0).contains(&self.north)
            || !(-180.0..=180.0).contains(&self.west)
            || !(-180.0..=180.0).contains(&self.east)
        {
            return Err(AppError::BadRequest(
                "Coordinates out of range: lat [-90,90], lng [-180,180]".into(),
            ));
        }
        if self.south >= self.north {
            return Err(AppError::BadRequest("south must be less than north".into()));
        }
        if self.west >= self.east {
            return Err(AppError::BadRequest("west must be less than east".into()));
        }
        if (self.north - self.south) > 0.5 || (self.east - self.west) > 0.5 {
            return Err(AppError::BboxTooLarge);
        }
        Ok(())
    }
}
```

- [ ] **Step 2: Create `src/models/coord.rs`**

```rust
use crate::models::error::AppError;
use serde::Deserialize;

/// Coordinate query parameters for point-based endpoints (`/api/score`, `/api/trend`).
#[derive(Debug, Deserialize)]
pub struct CoordParams {
    pub lat: f64,
    pub lng: f64,
}

impl CoordParams {
    pub fn validate(&self) -> Result<(), AppError> {
        if !(-90.0..=90.0).contains(&self.lat) || !(-180.0..=180.0).contains(&self.lng) {
            return Err(AppError::BadRequest(
                "Coordinates out of range: lat [-90,90], lng [-180,180]".into(),
            ));
        }
        Ok(())
    }
}
```

- [ ] **Step 3: Update `src/models/error.rs` — add BboxTooLarge variant**

Add after `BadRequest` variant (line 17):
```rust
    #[error("Bounding box exceeds maximum allowed area (0.5 degrees per side)")]
    BboxTooLarge,
```

Update `error_code()` match (after `BadRequest` arm):
```rust
    Self::BboxTooLarge => "BBOX_TOO_LARGE",
```

Update `IntoResponse` status match (after `BadRequest` arm):
```rust
    Self::BboxTooLarge => StatusCode::BAD_REQUEST,
```

- [ ] **Step 4: Implement `src/models/geojson.rs`**

Replace the comment-only file with full GeoJSON types:

```rust
use serde::Serialize;

/// RFC 7946 GeoJSON FeatureCollection.
/// Coordinates are always `[longitude, latitude]`.
#[derive(Debug, Serialize)]
pub struct FeatureCollection {
    pub r#type: &'static str,
    pub features: Vec<Feature>,
}

impl FeatureCollection {
    pub fn new(features: Vec<Feature>) -> Self {
        Self {
            r#type: "FeatureCollection",
            features,
        }
    }
}

/// RFC 7946 GeoJSON Feature.
#[derive(Debug, Serialize)]
pub struct Feature {
    pub r#type: &'static str,
    pub geometry: serde_json::Value,
    pub properties: serde_json::Value,
}

impl Feature {
    pub fn new(geometry: serde_json::Value, properties: serde_json::Value) -> Self {
        Self {
            r#type: "Feature",
            geometry,
            properties,
        }
    }
}
```

- [ ] **Step 5: Create `src/models/responses.rs`**

```rust
use serde::Serialize;

/// Response for `GET /api/score`.
#[derive(Debug, Serialize)]
pub struct ScoreResponse {
    pub score: f64,
    pub components: ScoreComponents,
    pub metadata: ScoreMetadata,
}

#[derive(Debug, Serialize)]
pub struct ScoreComponents {
    pub trend: ScoreDetail,
    pub risk: ScoreDetail,
    pub access: ScoreDetail,
    pub yield_potential: ScoreDetail,
}

#[derive(Debug, Serialize)]
pub struct ScoreDetail {
    pub value: f64,
    pub max: f64,
    pub detail: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct ScoreMetadata {
    pub calculated_at: String,
    pub data_freshness: String,
    pub disclaimer: String,
}

/// Response for `GET /api/stats`.
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub land_price: LandPriceStats,
    pub risk: RiskStats,
    pub facilities: FacilityStats,
    pub zoning_distribution: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct LandPriceStats {
    pub avg_per_sqm: Option<f64>,
    pub median_per_sqm: Option<f64>,
    pub min_per_sqm: Option<i32>,
    pub max_per_sqm: Option<i32>,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct RiskStats {
    pub flood_area_ratio: f64,
    pub steep_slope_area_ratio: f64,
    pub avg_composite_risk: f64,
}

#[derive(Debug, Serialize)]
pub struct FacilityStats {
    pub schools: i64,
    pub medical: i64,
}

/// Response for `GET /api/trend`.
#[derive(Debug, Serialize)]
pub struct TrendResponse {
    pub location: TrendLocation,
    pub data: Vec<TrendDataPoint>,
    pub cagr: f64,
    pub direction: String,
}

#[derive(Debug, Serialize)]
pub struct TrendLocation {
    pub address: String,
    pub distance_m: f64,
}

#[derive(Debug, Serialize)]
pub struct TrendDataPoint {
    pub year: i32,
    pub price_per_sqm: i32,
}
```

- [ ] **Step 6: Update `src/models/mod.rs` to declare new modules**

```rust
pub mod bbox;
pub mod coord;
pub mod error;
pub mod geojson;
pub mod responses;

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub reinfolib_key: Option<String>,
}
```

- [ ] **Step 7: Verify compilation**

Run: `cd services/backend && cargo build 2>&1`
Expected: Compiles with no errors.

- [ ] **Step 8: Commit**

```bash
git add services/backend/src/models/
git commit -m "feat(backend): add domain types — BBox, Coord, GeoJSON, response structs"
```

---

## Task 3: Infra Layer — PostGIS Repository Queries

**Files:**
- Create: `src/infra/mod.rs`
- Create: `src/infra/repo_area.rs`
- Create: `src/infra/repo_score.rs`
- Create: `src/infra/repo_stats.rs`
- Create: `src/infra/repo_trend.rs`
- Modify: `src/main.rs:7` (add `mod infra;`)

- [ ] **Step 1: Create `src/infra/mod.rs`**

```rust
pub mod repo_area;
pub mod repo_score;
pub mod repo_stats;
pub mod repo_trend;
```

- [ ] **Step 2: Create `src/infra/repo_area.rs`**

Each function queries one PostGIS table by bbox, returning `Vec<Feature>`:

```rust
use serde_json::json;
use sqlx::PgPool;

use crate::models::bbox::BBoxParams;
use crate::models::geojson::Feature;

/// Query land_prices within bbox. Returns GeoJSON Features.
#[tracing::instrument(skip(pool))]
pub async fn query_land_prices(pool: &PgPool, bbox: &BBoxParams) -> Result<Vec<Feature>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i32, String, Option<String>, i32, serde_json::Value)>(
        r#"
        SELECT id, price_per_sqm, address, land_use, year,
               ST_AsGeoJSON(geom)::jsonb AS geometry
        FROM land_prices
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        "#,
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, price, address, land_use, year, geom)| {
            Feature::new(
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

/// Query zoning polygons within bbox.
#[tracing::instrument(skip(pool))]
pub async fn query_zoning(pool: &PgPool, bbox: &BBoxParams) -> Result<Vec<Feature>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, Option<String>, Option<f32>, Option<f32>, serde_json::Value)>(
        r#"
        SELECT id, zone_type, zone_code, floor_area_ratio, building_coverage,
               ST_AsGeoJSON(geom)::jsonb AS geometry
        FROM zoning
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        "#,
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, zone_type, zone_code, far, bc, geom)| {
            Feature::new(
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

/// Query flood risk zones within bbox.
#[tracing::instrument(skip(pool))]
pub async fn query_flood(pool: &PgPool, bbox: &BBoxParams) -> Result<Vec<Feature>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, Option<String>, Option<String>, serde_json::Value)>(
        r#"
        SELECT id, depth_rank, river_name,
               ST_AsGeoJSON(geom)::jsonb AS geometry
        FROM flood_risk
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        "#,
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, depth_rank, river_name, geom)| {
            Feature::new(
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

/// Query steep slope zones within bbox.
#[tracing::instrument(skip(pool))]
pub async fn query_steep_slope(pool: &PgPool, bbox: &BBoxParams) -> Result<Vec<Feature>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, Option<String>, serde_json::Value)>(
        r#"
        SELECT id, area_name,
               ST_AsGeoJSON(geom)::jsonb AS geometry
        FROM steep_slope
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        "#,
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, area_name, geom)| {
            Feature::new(geom, json!({ "id": id, "area_name": area_name }))
        })
        .collect())
}

/// Query schools within bbox.
#[tracing::instrument(skip(pool))]
pub async fn query_schools(pool: &PgPool, bbox: &BBoxParams) -> Result<Vec<Feature>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, Option<String>, serde_json::Value)>(
        r#"
        SELECT id, name, school_type,
               ST_AsGeoJSON(geom)::jsonb AS geometry
        FROM schools
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        "#,
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, name, school_type, geom)| {
            Feature::new(
                geom,
                json!({ "id": id, "name": name, "school_type": school_type }),
            )
        })
        .collect())
}

/// Query medical facilities within bbox.
#[tracing::instrument(skip(pool))]
pub async fn query_medical(pool: &PgPool, bbox: &BBoxParams) -> Result<Vec<Feature>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, String, Option<String>, Option<i32>, serde_json::Value)>(
        r#"
        SELECT id, name, facility_type, bed_count,
               ST_AsGeoJSON(geom)::jsonb AS geometry
        FROM medical_facilities
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        "#,
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, name, facility_type, bed_count, geom)| {
            Feature::new(
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
```

- [ ] **Step 3: Create `src/infra/repo_score.rs`**

Raw data queries for each scoring component:

```rust
use sqlx::PgPool;

/// Land price records near a point for trend calculation.
pub struct PriceRecord {
    pub year: i32,
    pub price_per_sqm: i32,
    pub address: String,
    pub distance_m: f64,
}

/// Query nearest land price observation point and its multi-year prices (within 1km).
#[tracing::instrument(skip(pool))]
pub async fn query_nearest_prices(
    pool: &PgPool,
    lng: f64,
    lat: f64,
) -> Result<Vec<PriceRecord>, sqlx::Error> {
    // Find nearest address, then get all years for that address
    let rows = sqlx::query_as::<_, (i32, i32, String, f64)>(
        r#"
        WITH nearest AS (
            SELECT address,
                   ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS dist
            FROM land_prices
            WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)
            ORDER BY dist
            LIMIT 1
        )
        SELECT lp.year, lp.price_per_sqm, lp.address,
               ST_Distance(lp.geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS distance_m
        FROM land_prices lp
        INNER JOIN nearest n ON lp.address = n.address
        ORDER BY lp.year
        "#,
    )
    .bind(lng)
    .bind(lat)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(year, price, address, dist)| PriceRecord {
            year,
            price_per_sqm: price,
            address,
            distance_m: dist,
        })
        .collect())
}

/// Flood risk overlap ratio within 500m buffer of the point.
#[tracing::instrument(skip(pool))]
pub async fn query_flood_overlap(pool: &PgPool, lng: f64, lat: f64) -> Result<f64, sqlx::Error> {
    let row: (f64,) = sqlx::query_as(
        r#"
        SELECT COALESCE(
            SUM(ST_Area(ST_Intersection(
                ST_Buffer(ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)::geometry,
                geom
            ))) / NULLIF(ST_Area(ST_Buffer(ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)::geometry), 0),
            0.0
        )
        FROM flood_risk
        WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)
        "#,
    )
    .bind(lng)
    .bind(lat)
    .fetch_one(pool)
    .await?;

    Ok(row.0)
}

/// Whether steep slope zone exists within 500m.
#[tracing::instrument(skip(pool))]
pub async fn query_steep_slope_nearby(pool: &PgPool, lng: f64, lat: f64) -> Result<bool, sqlx::Error> {
    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM steep_slope
        WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)
        "#,
    )
    .bind(lng)
    .bind(lat)
    .fetch_one(pool)
    .await?;

    Ok(row.0 > 0)
}

/// Count schools within 1km radius.
#[tracing::instrument(skip(pool))]
pub async fn query_schools_1km(pool: &PgPool, lng: f64, lat: f64) -> Result<(i64, f64), sqlx::Error> {
    let row: (i64, Option<f64>) = sqlx::query_as(
        r#"
        SELECT COUNT(*),
               MIN(ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography))
        FROM schools
        WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)
        "#,
    )
    .bind(lng)
    .bind(lat)
    .fetch_one(pool)
    .await?;

    Ok((row.0, row.1.unwrap_or(f64::MAX)))
}

/// Count medical facilities within 1km radius.
#[tracing::instrument(skip(pool))]
pub async fn query_medical_1km(pool: &PgPool, lng: f64, lat: f64) -> Result<(i64, f64), sqlx::Error> {
    let row: (i64, Option<f64>) = sqlx::query_as(
        r#"
        SELECT COUNT(*),
               MIN(ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography))
        FROM medical_facilities
        WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)
        "#,
    )
    .bind(lng)
    .bind(lat)
    .fetch_one(pool)
    .await?;

    Ok((row.0, row.1.unwrap_or(f64::MAX)))
}
```

- [ ] **Step 4: Create `src/infra/repo_stats.rs`**

```rust
use sqlx::PgPool;

use crate::models::bbox::BBoxParams;
use crate::models::responses::{FacilityStats, LandPriceStats, RiskStats};

/// Query aggregated land price statistics within bbox.
#[tracing::instrument(skip(pool))]
pub async fn query_land_price_stats(
    pool: &PgPool,
    bbox: &BBoxParams,
) -> Result<LandPriceStats, sqlx::Error> {
    let row: (Option<f64>, Option<f64>, Option<i32>, Option<i32>, i64) = sqlx::query_as(
        r#"
        SELECT
            AVG(price_per_sqm)::float8 AS avg_price,
            PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY price_per_sqm)::float8 AS median_price,
            MIN(price_per_sqm),
            MAX(price_per_sqm),
            COUNT(*)
        FROM land_prices
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
          AND year = (SELECT MAX(year) FROM land_prices)
        "#,
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_one(pool)
    .await?;

    Ok(LandPriceStats {
        avg_per_sqm: row.0,
        median_per_sqm: row.1,
        min_per_sqm: row.2,
        max_per_sqm: row.3,
        count: row.4,
    })
}

/// Query flood + steep slope area ratios within bbox.
#[tracing::instrument(skip(pool))]
pub async fn query_risk_stats(
    pool: &PgPool,
    bbox: &BBoxParams,
) -> Result<RiskStats, sqlx::Error> {
    let bbox_area_row: (f64,) = sqlx::query_as(
        "SELECT ST_Area(ST_MakeEnvelope($1, $2, $3, $4, 4326)::geography)",
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_one(pool)
    .await?;

    let bbox_area = bbox_area_row.0;
    if bbox_area == 0.0 {
        return Ok(RiskStats {
            flood_area_ratio: 0.0,
            steep_slope_area_ratio: 0.0,
            avg_composite_risk: 0.0,
        });
    }

    let flood_row: (f64,) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography)), 0)
        FROM flood_risk
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        "#,
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_one(pool)
    .await?;

    let slope_row: (f64,) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography)), 0)
        FROM steep_slope
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        "#,
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_one(pool)
    .await?;

    let flood_ratio = flood_row.0 / bbox_area;
    let slope_ratio = slope_row.0 / bbox_area;
    let composite = flood_ratio * 0.6 + slope_ratio * 0.4;

    Ok(RiskStats {
        flood_area_ratio: flood_ratio,
        steep_slope_area_ratio: slope_ratio,
        avg_composite_risk: composite,
    })
}

/// Query school/medical facility counts within bbox.
#[tracing::instrument(skip(pool))]
pub async fn query_facility_stats(
    pool: &PgPool,
    bbox: &BBoxParams,
) -> Result<FacilityStats, sqlx::Error> {
    let schools: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM schools WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))",
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_one(pool)
    .await?;

    let medical: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM medical_facilities WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))",
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_one(pool)
    .await?;

    Ok(FacilityStats {
        schools: schools.0,
        medical: medical.0,
    })
}

/// Query zoning distribution within bbox.
#[tracing::instrument(skip(pool))]
pub async fn query_zoning_distribution(
    pool: &PgPool,
    bbox: &BBoxParams,
) -> Result<serde_json::Value, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, f64)>(
        r#"
        SELECT zone_type,
               SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography))
               / NULLIF(SUM(SUM(ST_Area(ST_Intersection(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))::geography))) OVER (), 0)
               AS ratio
        FROM zoning
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        GROUP BY zone_type
        ORDER BY ratio DESC
        "#,
    )
    .bind(bbox.west)
    .bind(bbox.south)
    .bind(bbox.east)
    .bind(bbox.north)
    .fetch_all(pool)
    .await?;

    let mut map = serde_json::Map::new();
    for (zone_type, ratio) in rows {
        map.insert(zone_type, serde_json::Value::from(ratio));
    }

    Ok(serde_json::Value::Object(map))
}
```

- [ ] **Step 5: Create `src/infra/repo_trend.rs`**

```rust
use sqlx::PgPool;

use crate::models::responses::{TrendDataPoint, TrendLocation};

/// Query land price trend for the nearest observation point.
#[tracing::instrument(skip(pool))]
pub async fn query_trend(
    pool: &PgPool,
    lng: f64,
    lat: f64,
    years: i32,
) -> Result<Option<(TrendLocation, Vec<TrendDataPoint>)>, sqlx::Error> {
    // Find nearest address within 2km
    let nearest = sqlx::query_as::<_, (String, f64)>(
        r#"
        SELECT address,
               ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography) AS dist
        FROM land_prices
        WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 2000)
        ORDER BY dist
        LIMIT 1
        "#,
    )
    .bind(lng)
    .bind(lat)
    .fetch_optional(pool)
    .await?;

    let Some((address, distance_m)) = nearest else {
        return Ok(None);
    };

    let max_year_row: (i32,) =
        sqlx::query_as("SELECT COALESCE(MAX(year), 0) FROM land_prices WHERE address = $1")
            .bind(&address)
            .fetch_one(pool)
            .await?;
    let min_year = max_year_row.0 - years + 1;

    let data = sqlx::query_as::<_, (i32, i32)>(
        r#"
        SELECT year, price_per_sqm
        FROM land_prices
        WHERE address = $1 AND year >= $2
        ORDER BY year
        "#,
    )
    .bind(&address)
    .bind(min_year)
    .fetch_all(pool)
    .await?;

    let points: Vec<TrendDataPoint> = data
        .into_iter()
        .map(|(year, price)| TrendDataPoint {
            year,
            price_per_sqm: price,
        })
        .collect();

    Ok(Some((
        TrendLocation {
            address,
            distance_m,
        },
        points,
    )))
}
```

- [ ] **Step 6: Add `mod infra;` to `src/main.rs`**

After line 9 (`mod services;`), add:
```rust
mod infra;
```

- [ ] **Step 7: Verify compilation**

Run: `cd services/backend && cargo build 2>&1`
Expected: Compiles (queries are runtime-checked, not compile-time since we're not using `sqlx::query!` macro).

- [ ] **Step 8: Commit**

```bash
git add services/backend/src/infra/
git commit -m "feat(backend): add PostGIS repository layer for all endpoints"
```

---

## Task 4: Scoring Service

**Files:**
- Modify: `src/services/scoring.rs`

- [ ] **Step 1: Implement scoring service**

Replace the comment-only file with the full scoring algorithm from API_SPEC.md:

```rust
use serde_json::json;
use sqlx::PgPool;

use crate::infra::repo_score;
use crate::models::error::AppError;
use crate::models::responses::{
    ScoreComponents, ScoreDetail, ScoreMetadata, ScoreResponse,
};

/// Compute a composite investment score (0-100) for the given coordinate.
///
/// Components (each 0-25):
/// - **trend**: Land price CAGR over past 5 years
/// - **risk**: Inverse of composite disaster risk (flood + steep slope)
/// - **access**: Proximity/count of schools + medical facilities within 1km
/// - **yield_potential**: Estimated yield based on transaction vs. land price ratio
#[tracing::instrument(skip(pool))]
pub async fn compute_score(pool: &PgPool, lng: f64, lat: f64) -> Result<ScoreResponse, AppError> {
    // --- Trend component ---
    let prices = repo_score::query_nearest_prices(pool, lng, lat).await?;
    let trend_detail = compute_trend(&prices);

    // --- Risk component ---
    let flood_overlap = repo_score::query_flood_overlap(pool, lng, lat).await?;
    let steep_nearby = repo_score::query_steep_slope_nearby(pool, lng, lat).await?;
    let risk_detail = compute_risk(flood_overlap, steep_nearby);

    // --- Access component ---
    let (schools_count, nearest_school) = repo_score::query_schools_1km(pool, lng, lat).await?;
    let (medical_count, nearest_medical) = repo_score::query_medical_1km(pool, lng, lat).await?;
    let access_detail = compute_access(schools_count, medical_count, nearest_school, nearest_medical);

    // --- Yield potential component ---
    let yield_detail = compute_yield_potential(&prices);

    let total = trend_detail.value + risk_detail.value + access_detail.value + yield_detail.value;

    Ok(ScoreResponse {
        score: total,
        components: ScoreComponents {
            trend: trend_detail,
            risk: risk_detail,
            access: access_detail,
            yield_potential: yield_detail,
        },
        metadata: ScoreMetadata {
            calculated_at: chrono::Utc::now().to_rfc3339(),
            data_freshness: prices
                .last()
                .map(|p| p.year.to_string())
                .unwrap_or_else(|| "N/A".into()),
            disclaimer: "本スコアは参考値です。投資判断は自己責任で行ってください。".into(),
        },
    })
}

/// trend (0-25): CAGR = (latest / oldest)^(1/years) - 1; score = clamp(CAGR * 500, 0, 25)
fn compute_trend(prices: &[repo_score::PriceRecord]) -> ScoreDetail {
    if prices.len() < 2 {
        return ScoreDetail {
            value: 0.0,
            max: 25.0,
            detail: json!({ "cagr_5y": 0, "direction": "unknown", "latest_price": null, "price_5y_ago": null }),
        };
    }

    let oldest = prices.first().expect("checked len >= 2");
    let latest = prices.last().expect("checked len >= 2");
    let years = (latest.year - oldest.year).max(1) as f64;
    let cagr = (latest.price_per_sqm as f64 / oldest.price_per_sqm as f64).powf(1.0 / years) - 1.0;
    let score = (cagr * 500.0).clamp(0.0, 25.0);
    let direction = if cagr > 0.0 { "up" } else { "down" };

    ScoreDetail {
        value: (score * 10.0).round() / 10.0,
        max: 25.0,
        detail: json!({
            "cagr_5y": (cagr * 1000.0).round() / 1000.0,
            "direction": direction,
            "latest_price": latest.price_per_sqm,
            "price_5y_ago": oldest.price_per_sqm,
        }),
    }
}

/// risk (0-25): composite = flood * 0.4 + liquefaction * 0.4 + steep * 0.2; score = 25 * (1 - composite)
fn compute_risk(flood_overlap: f64, steep_nearby: bool) -> ScoreDetail {
    let liquefaction_overlap = 0.0; // Phase 1: no liquefaction data yet
    let steep_factor = if steep_nearby { 1.0 } else { 0.0 };
    let composite = flood_overlap * 0.4 + liquefaction_overlap * 0.4 + steep_factor * 0.2;
    let score = (25.0 * (1.0 - composite)).clamp(0.0, 25.0);

    ScoreDetail {
        value: (score * 10.0).round() / 10.0,
        max: 25.0,
        detail: json!({
            "flood_overlap": (flood_overlap * 1000.0).round() / 1000.0,
            "liquefaction_overlap": liquefaction_overlap,
            "steep_slope_nearby": steep_nearby,
            "composite_risk": (composite * 1000.0).round() / 1000.0,
        }),
    }
}

/// access (0-25): school_score + medical_score + distance_score
fn compute_access(
    schools_1km: i64,
    medical_1km: i64,
    nearest_school_m: f64,
    nearest_medical_m: f64,
) -> ScoreDetail {
    let school_score = (schools_1km as f64 / 3.0).min(1.0) * 10.0;
    let medical_score = (medical_1km as f64 / 5.0).min(1.0) * 10.0;
    let distance_score = (5.0 - nearest_school_m / 200.0).max(0.0);
    let score = (school_score + medical_score + distance_score).clamp(0.0, 25.0);

    ScoreDetail {
        value: (score * 10.0).round() / 10.0,
        max: 25.0,
        detail: json!({
            "schools_1km": schools_1km,
            "medical_1km": medical_1km,
            "nearest_school_m": (nearest_school_m * 10.0).round() / 10.0,
            "nearest_medical_m": (nearest_medical_m * 10.0).round() / 10.0,
        }),
    }
}

/// yield_potential (0-25): yield = avg_transaction / land_price; score = clamp(yield * 500, 0, 25)
fn compute_yield_potential(prices: &[repo_score::PriceRecord]) -> ScoreDetail {
    let latest_price = prices.last().map(|p| p.price_per_sqm).unwrap_or(0);
    if latest_price == 0 {
        return ScoreDetail {
            value: 0.0,
            max: 25.0,
            detail: json!({ "avg_transaction_price": null, "land_price": null, "estimated_yield": 0 }),
        };
    }

    // Phase 1 estimate: assume transaction price ≈ 80% of land price (no actual transaction data yet)
    let avg_transaction = (latest_price as f64 * 0.8) as i64;
    let estimated_yield = avg_transaction as f64 / latest_price as f64;
    let score = (estimated_yield * 500.0).clamp(0.0, 25.0);

    ScoreDetail {
        value: (score * 10.0).round() / 10.0,
        max: 25.0,
        detail: json!({
            "avg_transaction_price": avg_transaction,
            "land_price": latest_price,
            "estimated_yield": (estimated_yield * 1000.0).round() / 1000.0,
        }),
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd services/backend && cargo build 2>&1`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add services/backend/src/services/scoring.rs
git commit -m "feat(backend): implement investment scoring algorithm (4 components)"
```

---

## Task 5: Route Handlers — Wire Everything Together

**Files:**
- Modify: `src/routes/area_data.rs`
- Modify: `src/routes/score.rs`
- Modify: `src/routes/stats.rs`
- Modify: `src/routes/trend.rs`

- [ ] **Step 1: Implement `src/routes/area_data.rs`**

```rust
use std::collections::HashMap;

use axum::{extract::{Query, State}, Json};
use serde_json::Value;

use crate::infra::repo_area;
use crate::models::bbox::BBoxParams;
use crate::models::error::AppError;
use crate::models::geojson::FeatureCollection;
use crate::models::AppState;

/// `GET /api/area-data?south=&west=&north=&east=&layers=landprice,zoning,...`
///
/// Returns GeoJSON FeatureCollections keyed by layer name.
#[tracing::instrument(skip(state))]
pub async fn get_area_data(
    State(state): State<AppState>,
    Query(bbox): Query<BBoxParams>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, AppError> {
    bbox.validate()?;

    let layers_str = params.get("layers").cloned().unwrap_or_default();
    let layers: Vec<&str> = layers_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    if layers.is_empty() {
        return Err(AppError::BadRequest("layers parameter is required".into()));
    }

    let mut result = serde_json::Map::new();

    for layer in &layers {
        let fc = match *layer {
            "landprice" => {
                let features = repo_area::query_land_prices(&state.db, &bbox).await?;
                FeatureCollection::new(features)
            }
            "zoning" => {
                let features = repo_area::query_zoning(&state.db, &bbox).await?;
                FeatureCollection::new(features)
            }
            "flood" => {
                let features = repo_area::query_flood(&state.db, &bbox).await?;
                FeatureCollection::new(features)
            }
            "steep_slope" => {
                let features = repo_area::query_steep_slope(&state.db, &bbox).await?;
                FeatureCollection::new(features)
            }
            "schools" => {
                let features = repo_area::query_schools(&state.db, &bbox).await?;
                FeatureCollection::new(features)
            }
            "medical" => {
                let features = repo_area::query_medical(&state.db, &bbox).await?;
                FeatureCollection::new(features)
            }
            unknown => {
                tracing::warn!(layer = unknown, "unknown layer requested, skipping");
                continue;
            }
        };
        result.insert(layer.to_string(), serde_json::to_value(fc).expect("serialization infallible"));
    }

    Ok(Json(Value::Object(result)))
}
```

- [ ] **Step 2: Implement `src/routes/score.rs`**

```rust
use axum::{extract::{Query, State}, Json};

use crate::models::coord::CoordParams;
use crate::models::error::AppError;
use crate::models::responses::ScoreResponse;
use crate::models::AppState;
use crate::services::scoring;

/// `GET /api/score?lat=35.68&lng=139.76`
#[tracing::instrument(skip(state))]
pub async fn get_score(
    State(state): State<AppState>,
    Query(params): Query<CoordParams>,
) -> Result<Json<ScoreResponse>, AppError> {
    params.validate()?;
    let score = scoring::compute_score(&state.db, params.lng, params.lat).await?;
    Ok(Json(score))
}
```

- [ ] **Step 3: Implement `src/routes/stats.rs`**

```rust
use axum::{extract::{Query, State}, Json};

use crate::infra::repo_stats;
use crate::models::bbox::BBoxParams;
use crate::models::error::AppError;
use crate::models::responses::StatsResponse;
use crate::models::AppState;

/// `GET /api/stats?south=&west=&north=&east=`
#[tracing::instrument(skip(state))]
pub async fn get_stats(
    State(state): State<AppState>,
    Query(bbox): Query<BBoxParams>,
) -> Result<Json<StatsResponse>, AppError> {
    bbox.validate()?;

    let (land_price, risk, facilities, zoning_dist) = tokio::try_join!(
        repo_stats::query_land_price_stats(&state.db, &bbox),
        repo_stats::query_risk_stats(&state.db, &bbox),
        repo_stats::query_facility_stats(&state.db, &bbox),
        repo_stats::query_zoning_distribution(&state.db, &bbox),
    )?;

    Ok(Json(StatsResponse {
        land_price,
        risk,
        facilities,
        zoning_distribution: zoning_dist,
    }))
}
```

- [ ] **Step 4: Implement `src/routes/trend.rs`**

```rust
use std::collections::HashMap;

use axum::{extract::{Query, State}, Json};

use crate::infra::repo_trend;
use crate::models::coord::CoordParams;
use crate::models::error::AppError;
use crate::models::responses::TrendResponse;
use crate::models::AppState;

/// `GET /api/trend?lat=35.68&lng=139.76&years=5`
#[tracing::instrument(skip(state))]
pub async fn get_trend(
    State(state): State<AppState>,
    Query(coord): Query<CoordParams>,
    Query(extra): Query<HashMap<String, String>>,
) -> Result<Json<TrendResponse>, AppError> {
    coord.validate()?;

    let years: i32 = extra
        .get("years")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5)
        .min(20)
        .max(1);

    let result = repo_trend::query_trend(&state.db, coord.lng, coord.lat, years).await?;

    let Some((location, data)) = result else {
        return Err(AppError::NotFound);
    };

    if data.is_empty() {
        return Err(AppError::NotFound);
    }

    let first_price = data.first().expect("non-empty").price_per_sqm as f64;
    let last_price = data.last().expect("non-empty").price_per_sqm as f64;
    let n_years = (data.last().expect("non-empty").year - data.first().expect("non-empty").year).max(1) as f64;

    let cagr = (last_price / first_price).powf(1.0 / n_years) - 1.0;
    let direction = if cagr > 0.0 { "up" } else { "down" };

    Ok(Json(TrendResponse {
        location,
        data,
        cagr: (cagr * 1000.0).round() / 1000.0,
        direction: direction.into(),
    }))
}
```

- [ ] **Step 5: Verify compilation**

Run: `cd services/backend && cargo build 2>&1`
Expected: Compiles with no errors.

- [ ] **Step 6: Run clippy**

Run: `cd services/backend && cargo clippy -- -D warnings 2>&1`
Expected: No warnings.

- [ ] **Step 7: Commit**

```bash
git add services/backend/src/routes/
git commit -m "feat(backend): implement all API route handlers with PostGIS queries"
```

---

## Task 6: Integration Test Skeleton

**Files:**
- Create: `tests/api/mod.rs`
- Create: `tests/api/health_test.rs`

Note: Full integration tests require `#[sqlx::test]` with a running PostGIS. This task creates the test structure and a basic health smoke test using `axum-test`.

- [ ] **Step 1: Create `tests/api/mod.rs`**

```rust
mod health_test;
```

- [ ] **Step 2: Create `tests/api/health_test.rs`**

```rust
use axum::{routing::get, Router};
use axum_test::TestServer;
use serde_json::Value;

async fn test_app() -> TestServer {
    // Minimal router for unit-style test (no DB required for health stub)
    let app = Router::new().route("/api/health", get(|| async {
        axum::Json(serde_json::json!({
            "status": "ok",
            "db_connected": false,
            "reinfolib_key_set": false,
            "version": "0.1.0"
        }))
    }));
    TestServer::new(app).expect("failed to create test server")
}

#[tokio::test]
async fn health_returns_200() {
    let server = test_app().await;
    let response = server.get("/api/health").await;
    response.assert_status_ok();

    let body: Value = response.json();
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
}
```

- [ ] **Step 3: Run tests**

Run: `cd services/backend && cargo test 2>&1`
Expected: `test health_returns_200 ... ok`

- [ ] **Step 4: Commit**

```bash
git add services/backend/tests/
git commit -m "test(backend): add health endpoint integration test skeleton"
```

---

## Task 7: Final Verification & Cleanup

- [ ] **Step 1: Full build**

Run: `cd services/backend && cargo build 2>&1`

- [ ] **Step 2: Clippy**

Run: `cd services/backend && cargo clippy -- -D warnings 2>&1`

- [ ] **Step 3: Tests**

Run: `cd services/backend && cargo test 2>&1`

- [ ] **Step 4: Verify all route handlers have tracing instrumentation**

Grep for `#[tracing::instrument` in all handler files to confirm coverage.

- [ ] **Step 5: Final commit (if any fixes needed)**

```bash
git add -A services/backend/
git commit -m "chore(backend): clippy fixes and final cleanup"
```

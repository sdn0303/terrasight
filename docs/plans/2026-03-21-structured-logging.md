# Structured Logging Strategy Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace all format-string logs with structured key-value `tracing` fields, add appropriate logging at every layer following Clean Architecture conventions, and enforce consistent log levels.

**Architecture:** Logging occurs **once** per concern boundary. Handler layer logs request receipt and response; Usecase layer logs business decisions and outcomes; Infra layer logs external I/O (DB queries, HTTP calls) at `debug!` level. Errors are logged at the layer that handles them (not propagated + re-logged). The `api-core` `ApiError` already logs `warn!` on every error response — individual handlers must NOT duplicate this.

**Tech Stack:** `tracing` 0.1, `tracing-subscriber` 0.3, existing `realestate-telemetry` lib

---

## Log Level Policy

| Level | When to use | Examples |
|-------|-------------|---------|
| `error!` | System degradation requiring operator attention | DB connection failed, config missing, panic guard |
| `warn!` | Recoverable issues, client errors, retries | Validation failure (via ApiError), rate limited, retry attempt |
| `info!` | Lifecycle events, request summaries, business outcomes | Server started, request completed with result summary, score computed |
| `debug!` | Development diagnostics: queries, records, GeoJSON | Row counts, feature counts, raw query params, DB fetch results |
| `trace!` | Very verbose: serialized payloads, full GeoJSON | Individual feature data, full response bodies |

## Structured Field Conventions

All structured fields use `snake_case`. Common fields:

| Field | Type | Used in |
|-------|------|---------|
| `port` | u16 | Server startup |
| `endpoint` | &str | Handler layer |
| `lat`, `lng` | f64 | Coord-based requests |
| `south`, `west`, `north`, `east` | f64 | BBox-based requests |
| `layers` | &str | area-data layer list |
| `layer` | &str | Per-layer operations |
| `feature_count` | usize | GeoJSON results |
| `row_count` | usize | DB query results |
| `score` | f64 | Score computation |
| `elapsed_ms` | u128 | Operation timing |
| `error` | Display/Debug | Error contexts |
| `attempt` | u32 | Retry loops |
| `status` | u16 | HTTP status codes |
| `db_connected` | bool | Health checks |
| `years` | i32 | Trend parameters |
| `cagr` | f64 | Trend results |
| `direction` | &str | Trend results |

---

### Task 1: Fix main.rs Startup Logs

**Files:**
- Modify: `services/backend/src/main.rs:35-39` (startup log)
- Modify: `services/backend/src/main.rs:85` (listening log)

**Step 1: Replace format-string logs with structured fields**

Replace:
```rust
tracing::info!(
    port = config.port,
    db_pool_size = config.db_max_connections,
    "starting server"
);
```
With:
```rust
tracing::info!(
    port = config.port,
    db_pool_size = config.db_max_connections,
    reinfolib_key_set = config.reinfolib_api_key.is_some(),
    version = env!("CARGO_PKG_VERSION"),
    "server starting"
);
```

Replace:
```rust
tracing::info!("listening on {addr}");
```
With:
```rust
tracing::info!(addr = %addr, "server listening");
```

**Step 2: Run clippy**

Run: `cd services/backend && cargo clippy -- -D warnings`
Expected: PASS (no format-string logs remain in main.rs)

**Step 3: Commit**

```bash
git add services/backend/src/main.rs
git commit -m "refactor: structured startup logs in main.rs"
```

---

### Task 2: Add Handler Layer Logging

Handlers use `#[tracing::instrument]` already but need explicit info/debug logs for request parameters and result summaries.

**Files:**
- Modify: `services/backend/src/handler/area_data.rs`
- Modify: `services/backend/src/handler/score.rs`
- Modify: `services/backend/src/handler/stats.rs`
- Modify: `services/backend/src/handler/trend.rs`
- Modify: `services/backend/src/handler/health.rs`

**Step 1: Add structured logging to each handler**

`area_data.rs` — log request params and result feature counts:
```rust
#[tracing::instrument(skip(usecase), fields(endpoint = "area-data"))]
pub async fn get_area_data(
    State(usecase): State<Arc<GetAreaDataUsecase>>,
    Query(params): Query<AreaDataQuery>,
) -> Result<Json<Value>, AppError> {
    let (bbox, layers) = params.into_domain()?;
    tracing::debug!(
        south = bbox.south(),
        west = bbox.west(),
        north = bbox.north(),
        east = bbox.east(),
        layers = ?layers.iter().map(|l| l.as_str()).collect::<Vec<_>>(),
        "area-data request parsed"
    );

    let result = usecase.execute(&bbox, &layers).await?;

    let total_features: usize = result.values().map(|v| v.len()).sum();
    tracing::info!(
        feature_count = total_features,
        layer_count = result.len(),
        "area-data response ready"
    );

    let mut map = serde_json::Map::new();
    for (layer, features) in result {
        tracing::debug!(
            layer = layer.as_str(),
            feature_count = features.len(),
            "layer features loaded"
        );
        let fc = FeatureCollectionDto::new(features.into_iter().map(geo_feature_to_dto).collect());
        map.insert(
            layer.as_str().to_string(),
            serde_json::to_value(fc).expect("FeatureCollection serialization is infallible"),
        );
    }

    Ok(Json(Value::Object(map)))
}
```

`score.rs` — log coord and computed score:
```rust
#[tracing::instrument(skip(usecase), fields(endpoint = "score"))]
pub async fn get_score(
    State(usecase): State<Arc<ComputeScoreUsecase>>,
    Query(params): Query<CoordQuery>,
) -> Result<Json<ScoreResponse>, AppError> {
    let coord = params.into_domain()?;
    tracing::debug!(lat = coord.lat(), lng = coord.lng(), "score request parsed");

    let score = usecase.execute(&coord).await?;
    tracing::info!(score = score.total(), "score computed");

    Ok(Json(ScoreResponse::from(score)))
}
```

`stats.rs` — log bbox:
```rust
#[tracing::instrument(skip(usecase), fields(endpoint = "stats"))]
pub async fn get_stats(
    State(usecase): State<Arc<GetStatsUsecase>>,
    Query(params): Query<BBoxQuery>,
) -> Result<Json<StatsResponse>, AppError> {
    let bbox = params.into_domain()?;
    tracing::debug!(
        south = bbox.south(),
        west = bbox.west(),
        north = bbox.north(),
        east = bbox.east(),
        "stats request parsed"
    );

    let stats = usecase.execute(&bbox).await?;
    tracing::info!(
        land_price_count = stats.land_price.count,
        composite_risk = %format!("{:.3}", stats.risk.composite_risk),
        "stats computed"
    );

    Ok(Json(StatsResponse::from(stats)))
}
```

`trend.rs` — log coord, years, and result:
```rust
#[tracing::instrument(skip(usecase), fields(endpoint = "trend"))]
pub async fn get_trend(
    State(usecase): State<Arc<GetTrendUsecase>>,
    Query(params): Query<TrendQuery>,
) -> Result<Json<TrendResponse>, AppError> {
    let (coord, years) = params.into_domain()?;
    tracing::debug!(
        lat = coord.lat(),
        lng = coord.lng(),
        years = years,
        "trend request parsed"
    );

    let trend = usecase.execute(&coord, years).await?;
    tracing::info!(
        cagr = trend.cagr,
        direction = trend.direction.as_str(),
        data_points = trend.data.len(),
        "trend computed"
    );

    Ok(Json(TrendResponse::from(trend)))
}
```

`health.rs` — log health result:
```rust
#[tracing::instrument(skip(usecase), fields(endpoint = "health"))]
pub async fn health(State(usecase): State<Arc<CheckHealthUsecase>>) -> Json<HealthResponse> {
    let status = usecase.execute().await;
    tracing::info!(
        status = status.status,
        db_connected = status.db_connected,
        reinfolib_key_set = status.reinfolib_key_set,
        "health check"
    );
    Json(HealthResponse::from(status))
}
```

**Step 2: Run clippy and tests**

Run: `cd services/backend && cargo clippy -- -D warnings && cargo test`
Expected: All pass

**Step 3: Commit**

```bash
git add services/backend/src/handler/
git commit -m "feat: structured logging in handler layer"
```

---

### Task 3: Add Usecase Layer Logging

Usecase is the single business-logging boundary. Log business outcomes at `info!` and intermediate data at `debug!`.

**Files:**
- Modify: `services/backend/src/usecase/compute_score.rs`
- Modify: `services/backend/src/usecase/get_area_data.rs`
- Modify: `services/backend/src/usecase/get_stats.rs`
- Modify: `services/backend/src/usecase/get_trend.rs`
- Modify: `services/backend/src/usecase/check_health.rs`

**Step 1: Add logging to each usecase**

`compute_score.rs` — log prices count and component scores:
```rust
pub async fn execute(&self, coord: &Coord) -> Result<InvestmentScore, DomainError> {
    let (prices, flood_overlap, steep_nearby, (schools_count, nearest_school), (medical_count, nearest_medical)) =
        tokio::try_join!(
            self.score_repo.find_nearest_prices(coord),
            self.score_repo.calc_flood_overlap(coord),
            self.score_repo.has_steep_slope_nearby(coord),
            self.score_repo.count_schools_nearby(coord),
            self.score_repo.count_medical_nearby(coord),
        )?;

    tracing::debug!(
        price_records = prices.len(),
        flood_overlap = %format!("{:.3}", flood_overlap),
        steep_nearby = steep_nearby,
        schools_count = schools_count,
        medical_count = medical_count,
        "score input data fetched"
    );

    let trend = compute_trend(&prices);
    let risk = compute_risk(flood_overlap, steep_nearby);
    let access = compute_access(schools_count, medical_count, nearest_school, nearest_medical);
    let yield_potential = compute_yield_potential(&prices);

    // ... rest unchanged ...
}
```

`get_area_data.rs` — log layer fetch results:
```rust
// Inside the async move block for each layer:
let features = match layer {
    // ... same match arms ...
}?;
tracing::debug!(
    layer = layer.as_str(),
    row_count = features.len(),
    "layer query completed"
);
Ok::<_, DomainError>((layer, features))
```

`get_stats.rs` — log individual stat computation:
```rust
pub async fn execute(&self, bbox: &BBox) -> Result<AreaStats, DomainError> {
    let (land_price, risk, facilities, zoning_distribution) = tokio::try_join!(
        self.stats_repo.calc_land_price_stats(bbox),
        self.stats_repo.calc_risk_stats(bbox),
        self.stats_repo.count_facilities(bbox),
        self.stats_repo.calc_zoning_distribution(bbox),
    )?;

    tracing::debug!(
        land_price_count = land_price.count,
        schools = facilities.schools,
        medical = facilities.medical,
        zoning_types = zoning_distribution.len(),
        "stats queries completed"
    );

    Ok(AreaStats { land_price, risk, facilities, zoning_distribution })
}
```

`get_trend.rs` — log trend data:
```rust
pub async fn execute(&self, coord: &Coord, years: i32) -> Result<TrendAnalysis, DomainError> {
    let years = years.clamp(1, 20);
    let result = self.trend_repo.find_trend(coord, years).await?;
    let (location, data) = result.ok_or(DomainError::NotFound)?;

    if data.is_empty() {
        tracing::debug!("no trend data found for nearest observation point");
        return Err(DomainError::NotFound);
    }

    tracing::debug!(
        address = %location.address,
        distance_m = %format!("{:.1}", location.distance_m),
        data_points = data.len(),
        "trend data loaded"
    );

    // ... rest unchanged ...
}
```

`check_health.rs` — log DB connectivity result:
```rust
pub async fn execute(&self) -> HealthStatus {
    let db_connected = self.health_repo.check_connection().await;
    if !db_connected {
        tracing::error!(db_connected = false, "database health check failed");
    }

    HealthStatus {
        status: if db_connected { "ok" } else { "degraded" },
        db_connected,
        reinfolib_key_set: self.reinfolib_key_set,
        version: env!("CARGO_PKG_VERSION"),
    }
}
```

**Step 2: Run clippy and tests**

Run: `cd services/backend && cargo clippy -- -D warnings && cargo test`
Expected: All pass

**Step 3: Commit**

```bash
git add services/backend/src/usecase/
git commit -m "feat: structured logging in usecase layer"
```

---

### Task 4: Add Infra Layer Debug Logging

Infra repos log DB interactions at `debug!` level: row counts from queries, and `error!` for unexpected DB failures before mapping to DomainError.

**Files:**
- Modify: `services/backend/src/infra/mod.rs` (enhance `map_db_err` with error logging)
- Modify: `services/backend/src/infra/pg_area_repository.rs`
- Modify: `services/backend/src/infra/pg_score_repository.rs`
- Modify: `services/backend/src/infra/pg_stats_repository.rs`
- Modify: `services/backend/src/infra/pg_trend_repository.rs`
- Modify: `services/backend/src/infra/pg_health_repository.rs`

**Step 1: Add error logging to `map_db_err`**

`infra/mod.rs` — log the raw sqlx error before converting:
```rust
pub(crate) fn map_db_err(e: sqlx::Error) -> crate::domain::error::DomainError {
    tracing::error!(error = %e, "database query failed");
    crate::domain::error::DomainError::Database(realestate_db::error::map_db_err(e).into_message())
}
```

**Step 2: Add debug logging to infra repositories**

The `#[tracing::instrument]` already emits spans. Add `debug!` for result row counts after each `fetch_all`/`fetch_one`. Example for `pg_area_repository.rs find_land_prices`:

After `let rows = bind_bbox(query, ...).fetch_all(&self.pool).await.map_err(map_db_err)?;`:
```rust
tracing::debug!(row_count = rows.len(), "land_prices query completed");
```

Apply the same pattern to each infra method:
- `pg_area_repository.rs`: Add `tracing::debug!(row_count = rows.len(), "TABLE query completed")` after each `fetch_all`
- `pg_score_repository.rs`: Add `tracing::debug!(row_count = rows.len(), "nearest_prices query completed")` after `find_nearest_prices` `fetch_all`. Other single-row fetches get no extra logging (the `#[tracing::instrument]` span is sufficient).
- `pg_stats_repository.rs`: Add `tracing::debug!` after each stat query with the relevant field name and value
- `pg_trend_repository.rs`: Add `tracing::debug!` after the nearest-point and data fetches
- `pg_health_repository.rs`: Add `tracing::debug!` for health check result

**Step 3: Run clippy and tests**

Run: `cd services/backend && cargo clippy -- -D warnings && cargo test`
Expected: All pass

**Step 4: Commit**

```bash
git add services/backend/src/infra/
git commit -m "feat: structured debug logging in infra layer"
```

---

### Task 5: Enhance mlit-client Logging

**Files:**
- Modify: `services/backend/lib/mlit-client/src/reinfolib.rs`

**Step 1: Add structured logging to reinfolib client**

In `fetch_tile_features` — log tile count and results:
```rust
async fn fetch_tile_features(
    &self,
    endpoint: &str,
    west: f64, south: f64, east: f64, north: f64,
    extra_params: &[(&str, &str)],
) -> Result<Vec<serde_json::Value>, MlitError> {
    let tiles = crate::tile::bbox_to_tiles(west, south, east, north, Self::DEFAULT_ZOOM);
    tracing::debug!(
        endpoint = endpoint,
        tile_count = tiles.len(),
        zoom = Self::DEFAULT_ZOOM,
        "fetching tile features"
    );

    let mut all_features = Vec::new();

    for tile in &tiles {
        // ... existing request logic ...

        tracing::debug!(
            endpoint = endpoint,
            z = tile.z,
            x = tile.x,
            y = tile.y,
            feature_count = geojson.features.len(),
            "tile response received"
        );

        // ... existing feature collection ...
    }

    tracing::debug!(
        endpoint = endpoint,
        total_features = all_features.len(),
        "tile features merged"
    );

    Ok(all_features)
}
```

In `request_with_retry` — improve existing retry logs:
```rust
// The existing warn! logs for 429 and transport errors are already well-structured.
// No changes needed — they already use attempt = and delay_secs =.
```

**Step 2: Run clippy and tests**

Run: `cd services/backend && cargo clippy --workspace -- -D warnings && cargo test --workspace`
Expected: All pass

**Step 3: Commit**

```bash
git add services/backend/lib/mlit-client/src/reinfolib.rs
git commit -m "feat: structured debug logging in reinfolib client"
```

---

### Task 6: Enhance api-core Error Logging

**Files:**
- Modify: `services/backend/lib/api-core/src/error.rs:88-92`

**Step 1: Add error message to the warn log**

The existing `ApiError::into_response` logs status and error_code but not the human-readable message. Add it:

```rust
tracing::warn!(
    status = status.as_u16(),
    error_code = self.0.error_code(),
    error_message = %self.0,
    "API error response"
);
```

**Step 2: Run clippy and tests**

Run: `cd services/backend && cargo clippy --workspace -- -D warnings && cargo test --workspace`
Expected: All pass

**Step 3: Commit**

```bash
git add services/backend/lib/api-core/src/error.rs
git commit -m "feat: include error_message in API error log"
```

---

### Task 7: Update Default Log Filter

**Files:**
- Modify: `services/backend/lib/telemetry/src/log/logger.rs:34`

**Step 1: Refine the default RUST_LOG filter**

Update the DEFAULT_FILTER to be more precise about which crates get which levels:

```rust
const DEFAULT_FILTER: &str = "\
    realestate_api=info,\
    realestate_api_core=info,\
    realestate_db=debug,\
    realestate_telemetry=info,\
    realestate_geo_math=debug,\
    mlit_client=info,\
    sqlx=warn,\
    tower_http=debug,\
    hyper=warn\
";
```

This ensures:
- Application crates log at `info` (handler + usecase summaries visible)
- DB and geo-math libs at `debug` (row counts, spatial calculations visible during development)
- SQLx internal chatter at `warn` (only errors)
- tower-http at `debug` (request/response tracing from the middleware)
- hyper at `warn` (suppress connection-level noise)

**Step 2: Run clippy and tests**

Run: `cd services/backend && cargo clippy --workspace -- -D warnings && cargo test --workspace`
Expected: All pass

**Step 3: Commit**

```bash
git add services/backend/lib/telemetry/src/log/logger.rs
git commit -m "refactor: fine-grained default RUST_LOG filter per crate"
```

---

### Task 8: Final Verification

**Step 1: Run full workspace clippy + tests**

Run: `cd services/backend && cargo clippy --workspace -- -D warnings && cargo test --workspace`
Expected: All clippy clean, all tests pass

**Step 2: Grep for remaining format-string log patterns**

Run: `grep -rn 'tracing::\(info\|warn\|error\|debug\)!("' services/backend/src/`
Expected: Zero matches (no format-string-only logs in application code). The telemetry lib's `warn!` for re-init is acceptable as it logs a framework error, not application data.

**Step 3: Verify no `println!` or `eprintln!` remain**

Run: `grep -rn 'println!\|eprintln!' services/backend/src/`
Expected: Zero matches

**Step 4: Commit (if any fixups needed)**

```bash
git add -A
git commit -m "chore: final structured logging cleanup"
```

---

## Summary of Changes

| Layer | Log Level | What Gets Logged |
|-------|-----------|-----------------|
| `main.rs` | `info!` | Server start (port, pool size, version, key status), listening address |
| Handler | `debug!` | Parsed request params (bbox coords, lat/lng, layers) |
| Handler | `info!` | Response summary (feature_count, score, cagr, direction) |
| Usecase | `debug!` | Fetched data summaries (row counts, component inputs) |
| Usecase | `error!` | DB health check failure |
| Infra | `debug!` | Query row counts per table |
| Infra | `error!` | Raw DB errors (via `map_db_err`) |
| api-core | `warn!` | Every API error response (status, code, message) |
| mlit-client | `debug!` | Tile counts, per-tile feature counts, merged totals |
| mlit-client | `warn!` | Rate limit retries, transport error retries |

## Design Decisions

1. **No duplicate error logging**: Errors propagate via `?` and are logged **once** at the boundary that handles them. `map_db_err` logs `error!` in infra; `ApiError::into_response` logs `warn!` at the HTTP boundary. Handlers do NOT log errors they return via `?`.

2. **`debug!` for data, `info!` for outcomes**: Raw row counts, coordinates, and query parameters are `debug!` (useful for development, too verbose for production). Business outcomes (score computed, features returned) are `info!`.

3. **`error!` is reserved for operator-actionable issues**: Only DB connection failures and unexpected system errors. Client validation errors are `warn!` (via `ApiError`), not `error!`.

4. **`#[tracing::instrument]` spans preserved**: All handler and infra methods keep their existing `#[tracing::instrument]` attributes for span context. The new explicit logs add structured fields within these spans.

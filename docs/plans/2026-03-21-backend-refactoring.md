# Backend Refactoring: Constants, DI, Exhaustive Matching

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Extract all magic numbers/hardcoded values to named constants, consolidate DI composition root into a struct, and replace if/else with match for exhaustive compiler-checked branching.

**Architecture:** Domain-layer constants module for business rules, infra-layer constants for spatial queries, DI struct wrapping all usecases for single-point wiring, match expressions where the compiler can enforce exhaustiveness.

**Tech Stack:** Rust, Axum, async_trait, sqlx, tokio

---

### Task 1: Create domain constants module for scoring business rules

**Files:**
- Create: `services/backend/src/domain/constants.rs`
- Modify: `services/backend/src/domain/mod.rs`

**Step 1: Write the constants module**

Create `services/backend/src/domain/constants.rs`:

```rust
//! Named constants for scoring algorithms and business rules.
//!
//! Centralised here so that:
//! - business rules are documented in one place
//! - the compiler catches typos (const vs magic literal)
//! - changing a threshold only requires editing one line

// ─── Score component maximums ────────────────────────
/// Each of the 4 score components (trend, risk, access, yield) is scored 0–MAX.
pub const SCORE_COMPONENT_MAX: f64 = 25.0;

// ─── Trend scoring ──────────────────────────────────
/// CAGR multiplier: score = clamp(CAGR * TREND_CAGR_MULTIPLIER, 0, SCORE_COMPONENT_MAX)
pub const TREND_CAGR_MULTIPLIER: f64 = 500.0;

// ─── Risk scoring (point-based) ─────────────────────
/// Weight for flood overlap in point-based risk score.
pub const RISK_WEIGHT_FLOOD: f64 = 0.4;
/// Weight for liquefaction overlap in point-based risk score.
/// Phase 1: liquefaction data not yet available (input = 0.0).
pub const RISK_WEIGHT_LIQUEFACTION: f64 = 0.4;
/// Weight for steep slope proximity in point-based risk score.
pub const RISK_WEIGHT_STEEP: f64 = 0.2;

// ─── Risk scoring (area-based stats) ────────────────
/// Weight for flood area ratio in area-based composite risk (stats endpoint).
/// Differs from point-based scoring intentionally: stats has no liquefaction layer.
pub const STATS_RISK_WEIGHT_FLOOD: f64 = 0.6;
/// Weight for steep slope area ratio in area-based composite risk (stats endpoint).
pub const STATS_RISK_WEIGHT_STEEP: f64 = 0.4;

// ─── Access scoring ─────────────────────────────────
/// Schools needed within 1km for maximum school sub-score.
pub const ACCESS_SCHOOL_SATURATION: f64 = 3.0;
/// Maximum sub-score from school count (out of SCORE_COMPONENT_MAX).
pub const ACCESS_SCHOOL_MAX_SCORE: f64 = 10.0;
/// Medical facilities needed within 1km for maximum medical sub-score.
pub const ACCESS_MEDICAL_SATURATION: f64 = 5.0;
/// Maximum sub-score from medical count (out of SCORE_COMPONENT_MAX).
pub const ACCESS_MEDICAL_MAX_SCORE: f64 = 10.0;
/// Distance divisor for nearest-school distance bonus.
pub const ACCESS_DISTANCE_DIVISOR: f64 = 200.0;
/// Maximum distance bonus (subtracted from this value).
pub const ACCESS_DISTANCE_MAX_BONUS: f64 = 5.0;

// ─── Yield scoring ──────────────────────────────────
/// Transaction-to-land-price ratio estimate (Phase 1 assumption).
pub const YIELD_TRANSACTION_RATIO: f64 = 0.8;
/// Yield-to-score multiplier: score = clamp(yield * YIELD_SCORE_MULTIPLIER, 0, MAX).
pub const YIELD_SCORE_MULTIPLIER: f64 = 500.0;

// ─── Spatial search radii (meters) ──────────────────
/// Buffer radius for flood/steep-slope risk assessment.
pub const RADIUS_RISK_BUFFER_M: f64 = 500.0;
/// Search radius for facility proximity (schools, medical, land prices for scoring).
pub const RADIUS_FACILITY_SEARCH_M: f64 = 1000.0;
/// Search radius for nearest trend observation point.
pub const RADIUS_TREND_SEARCH_M: f64 = 2000.0;

// ─── Trend analysis ─────────────────────────────────
/// Default number of years for trend analysis.
pub const TREND_DEFAULT_YEARS: i32 = 5;
/// Minimum years for trend analysis.
pub const TREND_MIN_YEARS: i32 = 1;
/// Maximum years for trend analysis.
pub const TREND_MAX_YEARS: i32 = 20;

// ─── Coordinate validation ──────────────────────────
/// Maximum latitude (WGS84).
pub const LAT_MAX: f64 = 90.0;
/// Maximum longitude (WGS84).
pub const LNG_MAX: f64 = 180.0;
/// Maximum bounding box side length in degrees.
pub const BBOX_MAX_SIDE_DEG: f64 = 0.5;

// ─── Decimal precision ──────────────────────────────
/// Decimal places for score component values.
pub const PRECISION_SCORE: u32 = 1;
/// Decimal places for ratios (CAGR, risk overlap, etc.).
pub const PRECISION_RATIO: u32 = 3;
/// Decimal places for distances in meters.
pub const PRECISION_DISTANCE: u32 = 1;

// ─── Formatting / display ───────────────────────────
/// Median percentile value for SQL PERCENTILE_CONT.
pub const MEDIAN_PERCENTILE: f64 = 0.5;
/// PostGIS SRID for WGS84.
pub const SRID_WGS84: i32 = 4326;

// ─── Health status strings ──────────────────────────
pub const HEALTH_STATUS_OK: &str = "ok";
pub const HEALTH_STATUS_DEGRADED: &str = "degraded";

// ─── User-facing text ───────────────────────────────
pub const SCORE_DISCLAIMER: &str =
    "本スコアは参考値です。投資判断は自己責任で行ってください。";
```

**Step 2: Register the module**

In `services/backend/src/domain/mod.rs`, add:
```rust
pub mod constants;
```

**Step 3: Run tests**

Run: `cd services/backend && cargo test --lib`
Expected: All existing tests pass (constants module has no tests yet, just declarations).

**Step 4: Commit**

```bash
git add services/backend/src/domain/constants.rs services/backend/src/domain/mod.rs
git commit -m "refactor: extract business rule constants to domain::constants"
```

---

### Task 2: Replace magic numbers in scoring usecases with constants

**Files:**
- Modify: `services/backend/src/usecase/compute_score.rs`
- Modify: `services/backend/src/usecase/get_trend.rs`
- Modify: `services/backend/src/usecase/check_health.rs`

**Step 1: Update compute_score.rs**

Replace all magic numbers with constants from `crate::domain::constants::*`:

- `25.0` → `SCORE_COMPONENT_MAX`
- `500.0` (CAGR multiplier) → `TREND_CAGR_MULTIPLIER`
- `0.4` (flood) → `RISK_WEIGHT_FLOOD`
- `0.4` (liquefaction) → `RISK_WEIGHT_LIQUEFACTION`
- `0.2` (steep) → `RISK_WEIGHT_STEEP`
- `3.0` → `ACCESS_SCHOOL_SATURATION`
- `10.0` (school) → `ACCESS_SCHOOL_MAX_SCORE`
- `5.0` (medical) → `ACCESS_MEDICAL_SATURATION`
- `10.0` (medical) → `ACCESS_MEDICAL_MAX_SCORE`
- `200.0` → `ACCESS_DISTANCE_DIVISOR`
- `5.0` (distance bonus) → `ACCESS_DISTANCE_MAX_BONUS`
- `0.8` → `YIELD_TRANSACTION_RATIO`
- `500.0` (yield multiplier) → `YIELD_SCORE_MULTIPLIER`
- `"N/A"` → `"N/A"` (keep as-is, it's a data absence marker not a business rule)

**Step 2: Update get_trend.rs**

- `years.clamp(1, 20)` → `years.clamp(TREND_MIN_YEARS, TREND_MAX_YEARS)`

**Step 3: Update check_health.rs**

- `"ok"` / `"degraded"` → `HEALTH_STATUS_OK` / `HEALTH_STATUS_DEGRADED`
- Replace `if db_connected { ... } else { ... }` with `match db_connected { true => ..., false => ... }`

**Step 4: Run tests**

Run: `cd services/backend && cargo test --lib`
Expected: All tests pass (behavior unchanged, only naming).

**Step 5: Run clippy**

Run: `cd services/backend && cargo clippy -- -D warnings`
Expected: Zero warnings.

**Step 6: Commit**

```bash
git add services/backend/src/usecase/
git commit -m "refactor: replace magic numbers in usecases with named constants"
```

---

### Task 3: Replace magic numbers in domain value objects

**Files:**
- Modify: `services/backend/src/domain/value_object.rs`

**Step 1: Update BBox::new and Coord::new validation**

Replace:
- `90.0` → `LAT_MAX`
- `180.0` → `LNG_MAX`
- `0.5` → `BBOX_MAX_SIDE_DEG`

Update the `BBoxTooLarge` error message in `domain/error.rs` to use the constant:
- `"Bounding box exceeds maximum allowed area (0.5 degrees per side)"` → use `format!` or keep as-is since DomainError is in domain layer (prefer keeping the string literal to avoid runtime formatting in a const context; document the relationship in a comment).

**Step 2: Run tests**

Run: `cd services/backend && cargo test --lib domain`
Expected: All domain tests pass.

**Step 3: Commit**

```bash
git add services/backend/src/domain/
git commit -m "refactor: use named constants for coordinate validation bounds"
```

---

### Task 4: Replace magic numbers in infrastructure layer

**Files:**
- Modify: `services/backend/src/infra/pg_stats_repository.rs`

**Step 1: Update risk weight in pg_stats_repository.rs**

Replace in `calc_risk_stats`:
- `flood_ratio * 0.6 + slope_ratio * 0.4` → `flood_ratio * STATS_RISK_WEIGHT_FLOOD + slope_ratio * STATS_RISK_WEIGHT_STEEP`

Note: SQL-embedded numeric literals (500, 1000, 2000, 4326, 0.5 in PERCENTILE_CONT) cannot easily use Rust constants since they are inside SQL string literals. Document with comments referencing the constant names instead of extracting them. Future improvement: parameterize these values or use `format!` with the constants.

**Step 2: Add comments to SQL queries referencing constants**

In `pg_score_repository.rs`, add comments above each SQL query:
```rust
// Search radius: RADIUS_FACILITY_SEARCH_M (1000m)
// Buffer radius: RADIUS_RISK_BUFFER_M (500m)
// SRID: SRID_WGS84 (4326)
```

In `pg_trend_repository.rs`:
```rust
// Search radius: RADIUS_TREND_SEARCH_M (2000m)
```

**Step 3: Run tests**

Run: `cd services/backend && cargo test --lib`
Expected: All tests pass.

**Step 4: Commit**

```bash
git add services/backend/src/infra/
git commit -m "refactor: use named constants for risk weights in stats repository"
```

---

### Task 5: Replace hardcoded strings in handler response

**Files:**
- Modify: `services/backend/src/handler/response.rs`
- Modify: `services/backend/src/handler/request.rs`

**Step 1: Update response.rs**

Replace disclaimer string:
- `"本スコアは参考値です。投資判断は自己責任で行ってください。".into()` → `SCORE_DISCLAIMER.to_string()`

**Step 2: Update request.rs**

Replace default_years:
- `fn default_years() -> i32 { 5 }` → `fn default_years() -> i32 { TREND_DEFAULT_YEARS }`

**Step 3: Run tests**

Run: `cd services/backend && cargo test --lib`
Expected: All tests pass.

**Step 4: Commit**

```bash
git add services/backend/src/handler/
git commit -m "refactor: use named constants in handler request/response"
```

---

### Task 6: Consolidate DI composition root into AppState struct

**Files:**
- Create: `services/backend/src/app_state.rs`
- Modify: `services/backend/src/main.rs`

**Step 1: Create AppState struct**

Create `services/backend/src/app_state.rs`:

```rust
use std::sync::Arc;

use sqlx::PgPool;

use crate::infra::pg_area_repository::PgAreaRepository;
use crate::infra::pg_health_repository::PgHealthRepository;
use crate::infra::pg_score_repository::PgScoreRepository;
use crate::infra::pg_stats_repository::PgStatsRepository;
use crate::infra::pg_trend_repository::PgTrendRepository;
use crate::usecase::check_health::CheckHealthUsecase;
use crate::usecase::compute_score::ComputeScoreUsecase;
use crate::usecase::get_area_data::GetAreaDataUsecase;
use crate::usecase::get_stats::GetStatsUsecase;
use crate::usecase::get_trend::GetTrendUsecase;

/// Composition root: wires Infra → Domain traits → Usecases.
///
/// All dependency injection happens here. Each usecase is wrapped in `Arc`
/// for shared ownership across Axum handler tasks.
pub struct AppState {
    pub health: Arc<CheckHealthUsecase>,
    pub area_data: Arc<GetAreaDataUsecase>,
    pub score: Arc<ComputeScoreUsecase>,
    pub stats: Arc<GetStatsUsecase>,
    pub trend: Arc<GetTrendUsecase>,
}

impl AppState {
    /// Build the full dependency graph from a database pool and config.
    pub fn new(pool: PgPool, reinfolib_key_set: bool) -> Self {
        Self {
            health: Arc::new(CheckHealthUsecase::new(
                Arc::new(PgHealthRepository::new(pool.clone())),
                reinfolib_key_set,
            )),
            area_data: Arc::new(GetAreaDataUsecase::new(Arc::new(PgAreaRepository::new(
                pool.clone(),
            )))),
            score: Arc::new(ComputeScoreUsecase::new(Arc::new(PgScoreRepository::new(
                pool.clone(),
            )))),
            stats: Arc::new(GetStatsUsecase::new(Arc::new(PgStatsRepository::new(
                pool.clone(),
            )))),
            trend: Arc::new(GetTrendUsecase::new(Arc::new(PgTrendRepository::new(pool)))),
        }
    }
}
```

**Step 2: Simplify main.rs**

Replace the manual DI wiring section with:

```rust
mod app_state;

use app_state::AppState;

// In main():
let state = AppState::new(pool, config.reinfolib_api_key.is_some());

let app = Router::new()
    .route("/api/health", get(handler::health::health))
    .with_state(state.health)
    .route("/api/area-data", get(handler::area_data::get_area_data))
    .with_state(state.area_data)
    .route("/api/score", get(handler::score::get_score))
    .with_state(state.score)
    .route("/api/stats", get(handler::stats::get_stats))
    .with_state(state.stats)
    .route("/api/trend", get(handler::trend::get_trend))
    .with_state(state.trend)
    .layer(response_time::response_time_layer())
    .layer(request_id::request_id_layer())
    .layer(realestate_telemetry::http::trace_layer())
    .layer(CorsLayer::permissive())
    .layer(CompressionLayer::new());
```

Remove all individual `use` imports for infra and usecase types from main.rs (they are now encapsulated in AppState).

**Step 3: Run tests and build**

Run: `cd services/backend && cargo build && cargo test --lib`
Expected: Build succeeds, all tests pass.

**Step 4: Commit**

```bash
git add services/backend/src/app_state.rs services/backend/src/main.rs
git commit -m "refactor: consolidate DI composition root into AppState struct"
```

---

### Task 7: Replace if/else with match for exhaustive patterns

**Files:**
- Modify: `services/backend/src/usecase/compute_score.rs`
- Modify: `services/backend/src/usecase/get_trend.rs`
- Modify: `services/backend/src/usecase/check_health.rs`

**Step 1: Update compute_score.rs — steep_factor**

Replace:
```rust
let steep_factor = if steep_nearby { 1.0 } else { 0.0 };
```
With:
```rust
let steep_factor = match steep_nearby {
    true => 1.0,
    false => 0.0,
};
```

Replace:
```rust
let direction = if cagr > 0.0 { "up" } else { "down" };
```
With (using f64 comparison — `match` on float isn't ergonomic, keep as-is since there's no enum to be exhaustive over. Instead, this is already correct for a continuous value. Skip this one.)

**Step 2: Update get_trend.rs — direction**

Replace:
```rust
let direction = if cagr > 0.0 {
    TrendDirection::Up
} else {
    TrendDirection::Down
};
```
With (same reasoning — f64 comparison, not an enum. Keep as-is. `match` on float requires a guard and doesn't add exhaustiveness. Skip.)

**Step 3: Update check_health.rs — status determination**

This was already handled in Task 2 (the `if db_connected` → `match db_connected`).

**Step 4: Verify no other if/else chains remain that could benefit from match**

Scan for `if ... else` patterns on booleans or enums. The `LayerType::parse` already uses `match`. The `AreaDataQuery::into_domain` uses `if parsed.is_none()` which is idiomatic filter_map — keep as-is.

**Step 5: Run tests and clippy**

Run: `cd services/backend && cargo test --lib && cargo clippy -- -D warnings`
Expected: All tests pass, zero clippy warnings.

**Step 6: Commit**

```bash
git add services/backend/src/usecase/
git commit -m "refactor: use match over if for boolean branching in scoring"
```

---

### Task 8: Final verification — full workspace build + test + clippy

**Files:** None (verification only)

**Step 1: Build entire workspace**

Run: `cd services/backend && cargo build`
Expected: Success with zero errors.

**Step 2: Run all tests**

Run: `cd services/backend && cargo test`
Expected: All unit tests + doc tests pass.

**Step 3: Run clippy**

Run: `cd services/backend && cargo clippy -- -D warnings`
Expected: Zero warnings.

**Step 4: Review diff for consistency**

Run: `git diff --stat HEAD~8..HEAD` (or count of commits)
Verify all changes are coherent and no magic numbers remain unaddressed.

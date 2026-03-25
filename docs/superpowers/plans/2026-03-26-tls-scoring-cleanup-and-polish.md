# TLS Scoring — Cleanup & Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the frontend↔backend API contract mismatch, remove legacy scoring code, and verify the TLS system works end-to-end.

**Architecture:** The backend sends `sub_scores`/`grade_label`/`weight_preset` in different shapes than the frontend expects (`sub`/`label`/`metadata.weight_preset`). We align by updating the **backend** to match the frontend's schema (design spec is the source of truth). Legacy 4-component scoring code is removed. No new DB tables needed — all required tables exist.

**Tech Stack:** Rust Axum (backend), Next.js 16 + Zod + TanStack Query (frontend), PostgreSQL + PostGIS

---

## API Contract Mismatches (must fix)

| # | Frontend expects | Backend sends | Fix location |
|---|-----------------|---------------|--------------|
| 1 | `axes.*.sub` (array) | `axes.*.sub_scores` (array) | Backend: rename field `sub_scores` → `sub` |
| 2 | `tls.label` | `tls.grade_label` | Backend: rename field `grade_label` → `label` |
| 3 | `metadata.weight_preset` (string) | `tls.weight_preset` (WeightPreset enum) | Backend: move to metadata, serialize as string |
| 4 | `location: {lat, lng}` | (missing) | Backend: add location field from handler coord |
| 5 | (not in schema) | `metadata.calculated_at` (string) | Frontend: add `calculated_at` to metadata schema |

---

## File Structure

**Backend files to modify:**
- `services/backend/src/handler/response.rs` — Fix TLS DTO shapes (mismatches #1-#4), delete legacy ScoreResponse types
- `services/backend/src/handler/score.rs` — Pass lat/lng into response construction (mismatch #4)
- `services/backend/src/app_state.rs` — Remove legacy score wiring
- `services/backend/src/domain/repository.rs` — Remove `ScoreRepository` trait
- `services/backend/src/domain/value_object.rs` — Remove `InvestmentScore`, `ScoreComponent`
- `services/backend/src/usecase.rs` — Remove `pub mod compute_score`
- `services/backend/src/infra.rs` — Remove `pub mod pg_score_repository`
- `services/backend/tests/api_integration.rs` — Update score test to TLS shape

**Backend files to delete:**
- `services/backend/src/usecase/compute_score.rs`
- `services/backend/src/infra/pg_score_repository.rs`

**Frontend files to modify:**
- `services/frontend/src/lib/schemas.ts` — Add `calculated_at` to metadata
- `services/frontend/src/__tests__/schemas.test.ts` — Add `calculated_at` to fixtures
- `services/frontend/src/__tests__/hooks.test.ts` — Add `calculated_at` to fixtures

---

### Task 1: Fix API contract — backend DTO alignment + integration test

**Files:**
- Modify: `services/backend/src/handler/response.rs:229-337`
- Modify: `services/backend/src/handler/score.rs:18-35`
- Modify: `services/backend/tests/api_integration.rs:183-203`

- [ ] **Step 1: Rename `sub_scores` → `sub` in `AxisDto`**

In `response.rs:258`, change the field name:
```rust
// Before
pub sub_scores: Vec<SubScoreDto>,
// After
pub sub: Vec<SubScoreDto>,
```

And in `axis_to_dto` function (~line 288):
```rust
// Before
sub_scores: axis.sub_scores.into_iter().map(sub_score_to_dto).collect(),
// After
sub: axis.sub_scores.into_iter().map(sub_score_to_dto).collect(),
```

- [ ] **Step 2: Rename `grade_label` → `label` in `TlsSummaryDto`**

In `response.rs:240`:
```rust
// Before
pub grade_label: &'static str,
// After
pub label: &'static str,
```

And in `From<TlsOutput>` (~line 325):
```rust
// Before
grade_label: t.grade.label(),
// After
label: t.grade.label(),
```

- [ ] **Step 3: Move `weight_preset` from `tls` to `metadata`**

Remove `weight_preset` field from `TlsSummaryDto` (delete line 241: `pub weight_preset: WeightPreset,`).

Remove the `weight_preset` line from the `TlsSummaryDto` constructor in `From<TlsOutput>` (~line 326).

Add `weight_preset` to `TlsMetadataDto`:
```rust
pub struct TlsMetadataDto {
    pub calculated_at: String,
    pub weight_preset: String,  // NEW — serialized from WeightPreset enum
    pub data_freshness: String,
    pub disclaimer: String,
}
```

In `From<TlsOutput>` metadata construction, add:
```rust
metadata: TlsMetadataDto {
    calculated_at: chrono::Utc::now().to_rfc3339(),
    weight_preset: serde_json::to_value(&t.weight_preset)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| "balance".to_string()),
    data_freshness: t.data_freshness,
    disclaimer: SCORE_DISCLAIMER.to_string(),
},
```

Note: `WeightPreset` derives `Serialize` with `#[serde(rename_all = "snake_case")]` so `serde_json::to_value` produces `"balance"`, `"investment"`, etc. The `unwrap_or_else` fallback is defensive but should never trigger.

Also remove the `WeightPreset` import from the top of `response.rs` line 6 (it's no longer used in DTO types directly — only via serde serialization).

- [ ] **Step 4: Add `location` field to `TlsResponse`**

Add a new DTO and field to `TlsResponse`:
```rust
#[derive(Debug, Serialize)]
pub struct LocationDto {
    pub lat: f64,
    pub lng: f64,
}

pub struct TlsResponse {
    pub location: LocationDto,  // NEW
    pub tls: TlsSummaryDto,
    pub axes: AxesDto,
    pub cross_analysis: CrossAnalysisDto,
    pub metadata: TlsMetadataDto,
}
```

Replace `impl From<TlsOutput> for TlsResponse` with a constructor that accepts lat/lng:
```rust
impl TlsResponse {
    pub fn new(lat: f64, lng: f64, t: TlsOutput) -> Self {
        Self {
            location: LocationDto { lat, lng },
            tls: TlsSummaryDto {
                score: t.score,
                grade: t.grade.as_str(),
                label: t.grade.label(),
            },
            axes: axes_to_dto(t.axes),
            cross_analysis: cross_analysis_to_dto(t.cross_analysis),
            metadata: TlsMetadataDto {
                calculated_at: chrono::Utc::now().to_rfc3339(),
                weight_preset: serde_json::to_value(&t.weight_preset)
                    .ok()
                    .and_then(|v| v.as_str().map(String::from))
                    .unwrap_or_else(|| "balance".to_string()),
                data_freshness: t.data_freshness,
                disclaimer: SCORE_DISCLAIMER.to_string(),
            },
        }
    }
}
```

- [ ] **Step 5: Update handler to use new constructor**

In `handler/score.rs:34`, change:
```rust
// Before (line 34)
Ok(Json(TlsResponse::from(output)))
// After
Ok(Json(TlsResponse::new(coord.lat(), coord.lng(), output)))
```

`coord` is still in scope at this point (created at line 22 via `params.into_domain()?`).

- [ ] **Step 6: Update integration test to TLS shape**

In `tests/api_integration.rs`, replace the test `score_returns_components_for_tokyo_station` (lines 183-203):
```rust
#[tokio::test]
async fn score_returns_tls_for_tokyo_station() {
    require_db!(server);

    let resp = server
        .get("/api/score")
        .add_query_param("lat", "35.681")
        .add_query_param("lng", "139.767")
        .await;

    resp.assert_status_ok();

    let body: Value = resp.json();
    // Location echo
    assert!(body["location"]["lat"].is_number(), "location.lat");
    assert!(body["location"]["lng"].is_number(), "location.lng");
    // TLS summary
    assert!(body["tls"]["score"].is_number(), "tls.score");
    assert!(body["tls"]["grade"].is_string(), "tls.grade");
    assert!(body["tls"]["label"].is_string(), "tls.label");
    // Axes use `sub` (not `sub_scores`)
    assert!(body["axes"]["disaster"]["sub"].is_array(), "axes.disaster.sub");
    assert!(body["axes"]["disaster"]["confidence"].is_number(), "confidence");
    // Metadata
    assert!(body["metadata"]["weight_preset"].is_string(), "weight_preset in metadata");
    assert!(body["metadata"]["calculated_at"].is_string(), "calculated_at");
    assert!(body["metadata"]["disclaimer"].is_string(), "disclaimer");
}
```

- [ ] **Step 7: Build and verify all tests**

Run: `cd services/backend && cargo fmt --all && cargo check && cargo clippy -- -D warnings && cargo test`
Expected: Clean compilation, zero warnings, all tests pass (11 integration tests, renamed test).

- [ ] **Step 8: Commit**

```bash
git add services/backend/src/handler/response.rs services/backend/src/handler/score.rs services/backend/tests/api_integration.rs
git commit -m "fix(backend): align TLS response DTO with frontend schema contract"
```

---

### Task 2: Add `calculated_at` to frontend schema

**Files:**
- Modify: `services/frontend/src/lib/schemas.ts:135-139`
- Modify: `services/frontend/src/__tests__/schemas.test.ts:54-58`
- Modify: `services/frontend/src/__tests__/hooks.test.ts:74-78`

- [ ] **Step 1: Add `calculated_at` to TlsResponse metadata schema**

In `schemas.ts`, update the metadata object (line 135-139):
```typescript
metadata: z.object({
    calculated_at: z.string(),  // ISO 8601 timestamp from backend
    weight_preset: z.string(),
    data_freshness: z.string(),
    disclaimer: z.string(),
}),
```

- [ ] **Step 2: Update test fixture in schemas.test.ts**

Add `calculated_at: "2026-03-25T10:00:00Z"` to metadata in both test blocks (~line 54 and ~line 75).

- [ ] **Step 3: Update test fixture in hooks.test.ts**

Add `calculated_at: "2026-03-25T10:00:00Z"` to `SCORE_FIXTURE.metadata` (~line 75).

- [ ] **Step 4: Verify**

Run: `cd services/frontend && pnpm tsc --noEmit && pnpm vitest run`
Expected: 13 test files, 119 tests passed.

- [ ] **Step 5: Commit**

```bash
git add services/frontend/src/lib/schemas.ts services/frontend/src/__tests__/schemas.test.ts services/frontend/src/__tests__/hooks.test.ts
git commit -m "fix(frontend): add calculated_at to TLS metadata schema"
```

---

### Task 3: Delete legacy scoring code — backend cleanup

**Files:**
- Delete: `services/backend/src/usecase/compute_score.rs`
- Delete: `services/backend/src/infra/pg_score_repository.rs`
- Modify: `services/backend/src/usecase.rs` — remove `pub mod compute_score;`
- Modify: `services/backend/src/infra.rs` — remove `pub mod pg_score_repository;`
- Modify: `services/backend/src/app_state.rs:11,17,73-75,101-104`
- Modify: `services/backend/src/handler/response.rs:32-95` — remove ScoreResponse types
- Modify: `services/backend/src/domain/repository.rs:23-41` — remove ScoreRepository trait
- Modify: `services/backend/src/domain/value_object.rs:173-195` — remove InvestmentScore, ScoreComponent (note: line 195 is closing `}` of ScoreComponent)

- [ ] **Step 1: Delete legacy usecase and repository files**

```bash
rm services/backend/src/usecase/compute_score.rs
rm services/backend/src/infra/pg_score_repository.rs
```

- [ ] **Step 2: Remove module declarations**

In `usecase.rs`: remove line `pub mod compute_score;`
In `infra.rs`: remove line `pub mod pg_score_repository;`

- [ ] **Step 3: Clean up app_state.rs**

Remove these lines/sections:
- Line 11: `use crate::infra::pg_score_repository::PgScoreRepository;`
- Line 17: `use crate::usecase::compute_score::ComputeScoreUsecase;`
- Lines 73-75: Comment + `let _legacy_score_repo = Arc::new(PgScoreRepository::new(pool.clone()));`
- Lines 101-104: Comment + `#[allow(dead_code)]` + `type _LegacyScoreUsecase = ComputeScoreUsecase;`

- [ ] **Step 4: Remove legacy response types from response.rs**

Delete lines 32-95 entirely: `ScoreResponse` struct, `ScoreComponentsDto`, `ScoreDetailDto`, `ScoreMetadataDto`, and `impl From<InvestmentScore> for ScoreResponse`.

- [ ] **Step 5: Remove ScoreRepository trait from repository.rs**

Delete lines 23-41: the `// ─── Score` comment block and entire `ScoreRepository` trait.

- [ ] **Step 6: Remove legacy value objects from value_object.rs**

Delete lines 173-195: `InvestmentScore` struct + `impl InvestmentScore` + `ScoreComponent` struct (including the closing `}` on line 195).

- [ ] **Step 7: Fix dangling imports**

Run `cargo check` and fix any compilation errors. Likely fixes:
- Remove `use crate::domain::scoring::tls::WeightPreset;` from `response.rs` if still present and unused after Task 1 changes
- The wildcard import `use crate::domain::value_object::*;` in `response.rs:7` will compile cleanly (deleted types just disappear from scope)

- [ ] **Step 8: Verify full test suite**

Run: `cd services/backend && cargo fmt --all && cargo check && cargo clippy -- -D warnings && cargo test`
Expected: Clean compile, zero warnings, all 11 tests pass (integration test was already updated in Task 1).

- [ ] **Step 9: Commit**

```bash
git add -u services/backend/
git commit -m "refactor(backend): remove legacy 4-component scoring system"
```

---

## Scope Boundary

The following are **explicitly out of scope** for this plan (separate future work):

- Sub-score drilldown UI component (expand axis to show sub-components)
- Cross-analysis panel UI (`value_discovery`, `demand_signal`, `ground_safety`)
- Data freshness display in UI
- Phase 3 data expansion (liquefaction, tsunami, transit, population)
- Weight preset switching UI
- WASM-based score computation
- Design doc update: `axes.*.sub` is implemented as array (not object/map as shown in design spec)

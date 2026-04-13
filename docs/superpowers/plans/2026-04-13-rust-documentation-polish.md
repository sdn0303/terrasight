# Rust Documentation Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bring all 6 Rust crates to consistent documentation quality following `proj-doc-*` rules in `.claude/rules/rust.md`. Every `.rs` file gets `//!` module docs, every public item gets `///` with `# Examples` where applicable, and intra-doc links replace hardcoded paths.

**Architecture:** Bottom-up approach — shared lib crates first (domain, geo, server, mlit), then application crates (terrasight-api, terrasight-wasm). Each task targets one crate, is independently executable by a subagent, and produces a single commit.

**Tech Stack:** Rust 1.94, rustdoc, `cargo test --doc`

**Branch:** Continue on `feature/workspace-restructure`

**Reference:** `.claude/rules/rust.md` section "Documentation (proj-doc-*)" defines all conventions.

---

## Documentation Standards Summary

Every file must follow these patterns (from `proj-doc-*` rules):

### Module-level (`//!`)
```rust
//! Brief one-line summary of this module's purpose.
//!
//! Detailed explanation: what types live here, how they relate,
//! why this module exists in the architecture.
```

### Item-level (`///`)
```rust
/// One-line summary (appears in search and module listings).
///
/// Detailed explanation: motivation, constraints, usage context.
/// Do NOT repeat type signatures.
///
/// # Examples
///
/// ```
/// # use crate_name::Type;
/// let result = Type::new(42);
/// assert_eq!(result.value(), 42);
/// ```
///
/// # Errors
///
/// Returns [`Error::Validation`] if input is invalid.
```

### Constants
```rust
/// Maximum allowed bounding box side length in degrees.
///
/// Prevents excessively large spatial queries that would overwhelm
/// the database. Corresponds to roughly 55 km at the equator.
pub const BBOX_MAX_SIDE_DEG: f64 = 0.5;
```

---

## Audit Summary (Current State)

| Crate | Files | Module `//!` | Item `///` coverage | Doc tests | Priority |
|-------|-------|-------------|-------------------|-----------|----------|
| terrasight-domain | 8 | 50% | 70% | 2 | P1 |
| terrasight-geo | 5 | 40% | 80% | 8 | P1 |
| terrasight-server | 20 | 35% | 65% | 20 | P2 |
| terrasight-mlit | 9 | 44% | 75% | 4 | P2 |
| terrasight-api | 70 | 15% | 40% | 0 | P0 (largest gap) |
| terrasight-wasm | 8 | 75% | 89% | 8 | P3 (mostly done) |

---

## Task 0: terrasight-domain (8 files)

**Files:**
- Modify: `services/backend/lib/domain/src/lib.rs`
- Modify: `services/backend/lib/domain/src/constants.rs`
- Modify: `services/backend/lib/domain/src/types.rs`
- Modify: `services/backend/lib/domain/src/scoring.rs`
- Modify: `services/backend/lib/domain/src/scoring/constants.rs`
- Modify: `services/backend/lib/domain/src/scoring/tls.rs`
- Modify: `services/backend/lib/domain/src/scoring/axis.rs`
- Modify: `services/backend/lib/domain/src/scoring/sub_scores.rs`

- [ ] **Step 1:** Read all 8 files to understand current doc state
- [ ] **Step 2:** Add crate-level `//!` to `lib.rs`:
  - One-line: "Shared domain types, scoring logic, and constants for the Terrasight platform."
  - Describe the 3 modules (constants, scoring, types) and their purpose
  - Explain Backend + WASM sharing model
  - Note: all types are `pub` because they cross crate boundaries
- [ ] **Step 3:** Add `//!` to `scoring.rs` explaining the 5-axis TLS scoring system, referencing the 4 sub-modules
- [ ] **Step 4:** In `constants.rs`, convert ALL inline comments to `///` doc comments. Group constants with section doc comments (`//! -- Layer IDs --` etc.). Each constant needs one line explaining its purpose and where it's used
- [ ] **Step 5:** In `types.rs`, add `//!` module doc. Add field-level `///` for every struct field (e.g., `avg_per_sqm`, `flood_area_ratio`). Add `# Examples` with `LandPriceStats::default()` usage
- [ ] **Step 6:** In `scoring/constants.rs`, ensure every `pub const` and `pub(crate) const` has `///` explaining the scoring semantics (what it means, not just its name). The weight tables (`S1_WEIGHTS`, etc.) need explanations of what each axis measures
- [ ] **Step 7:** In `scoring/tls.rs`, add `///` to `WeightPreset` variants (Balance, Investment, Residential, DisasterFocus) explaining what each emphasizes. Add `# Examples` to `compute_tls` and `compute_cross_analysis`. Add `# Errors` sections. Document `Grade` enum variants
- [ ] **Step 8:** In `scoring/axis.rs`, verify existing docs (already EXCELLENT per audit). Add intra-doc links to `[`constants`](super::constants)` references
- [ ] **Step 9:** In `scoring/sub_scores.rs`, add intra-doc links to referenced constants. Verify all `score_*` functions document their input range and output range (0-100)
- [ ] **Step 10:** Run `cd services/backend && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p terrasight-domain && cargo test --doc -p terrasight-domain`
- [ ] **Step 11:** Commit: `docs(domain): add comprehensive rustdoc to terrasight-domain`

---

## Task 1: terrasight-geo (5 files)

**Files:**
- Modify: `services/backend/lib/geo/src/lib.rs`
- Modify: `services/backend/lib/geo/src/spatial.rs`
- Modify: `services/backend/lib/geo/src/tile.rs`
- Modify: `services/backend/lib/geo/src/finance.rs`
- Modify: `services/backend/lib/geo/src/rounding.rs`

- [ ] **Step 1:** Read all 5 files
- [ ] **Step 2:** Add crate-level `//!` to `lib.rs`:
  - One-line: "Pure computation utilities for geospatial math, finance, and tiling."
  - List the 4 modules with one-line descriptions
  - Note: zero external dependencies (pure functions only)
- [ ] **Step 3:** In `spatial.rs`, add `//!` module doc. Document `LayerKind` enum variants (each variant needs `///` explaining what layer it represents and typical feature density). Document constants `MIN_FEATURE_LIMIT`, `MAX_FEATURE_LIMIT`, etc. Add `# Examples` to `compute_feature_limit` and `bbox_area_deg2`
- [ ] **Step 4:** In `tile.rs`, verify existing docs (EXCELLENT per audit). Add any missing intra-doc links
- [ ] **Step 5:** In `finance.rs`, add `//!` module doc. `compute_cagr` should have `# Examples` showing positive, negative, and zero growth
- [ ] **Step 6:** In `rounding.rs`, add `//!` module doc
- [ ] **Step 7:** Run `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p terrasight-geo && cargo test --doc -p terrasight-geo`
- [ ] **Step 8:** Commit: `docs(geo): add comprehensive rustdoc to terrasight-geo`

---

## Task 2: terrasight-server (20 files)

**Files:**
- Modify: `services/backend/lib/server/src/lib.rs` (already excellent — verify)
- Modify: `services/backend/lib/server/src/db.rs`
- Modify: `services/backend/lib/server/src/db/error.rs`
- Modify: `services/backend/lib/server/src/db/geo.rs`
- Modify: `services/backend/lib/server/src/db/pool.rs`
- Modify: `services/backend/lib/server/src/db/spatial.rs`
- Modify: `services/backend/lib/server/src/http.rs`
- Modify: `services/backend/lib/server/src/http/error.rs`
- Modify: `services/backend/lib/server/src/http/middleware.rs`
- Modify: `services/backend/lib/server/src/http/middleware/rate_limit.rs`
- Modify: `services/backend/lib/server/src/http/middleware/request_id.rs`
- Modify: `services/backend/lib/server/src/http/middleware/response_time.rs`
- Modify: `services/backend/lib/server/src/http/response.rs`
- Modify: `services/backend/lib/server/src/http/tracing.rs`
- Modify: `services/backend/lib/server/src/log.rs`
- Modify: `services/backend/lib/server/src/log/logger.rs`
- Modify: `services/backend/lib/server/src/metrics.rs`
- Modify: `services/backend/lib/server/src/metrics/names.rs`
- Modify: `services/backend/lib/server/src/metrics/tags.rs`

- [ ] **Step 1:** Read all module index files (`db.rs`, `http.rs`, `log.rs`, `http/middleware.rs`)
- [ ] **Step 2:** Add `//!` to `db.rs`: "Database infrastructure: connection pooling, error mapping, spatial query helpers, and GeoJSON conversion."
- [ ] **Step 3:** Add `//!` to `http.rs`: "HTTP infrastructure: error mapping, middleware (rate limit, response time, request ID), tracing, and GeoJSON response types."
- [ ] **Step 4:** Add `//!` to `log.rs`: "Structured logging initialization with format selection (JSON/Pretty)."
- [ ] **Step 5:** Add `//!` to `http/middleware.rs`: "Axum middleware layers for rate limiting, response time tracking, and request ID propagation."
- [ ] **Step 6:** Read all implementation files. For each file missing `//!`, add module-level doc explaining what it provides
- [ ] **Step 7:** For files already well-documented (http/error.rs, db/spatial.rs, middleware/*), add intra-doc links where plain-text references exist
- [ ] **Step 8:** In `metrics/names.rs` and `metrics/tags.rs`, ensure every constant has `///` explaining where/when it's recorded
- [ ] **Step 9:** Run `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p terrasight-server && cargo test --doc -p terrasight-server`
- [ ] **Step 10:** Commit: `docs(server): add comprehensive rustdoc to terrasight-server`

---

## Task 3: terrasight-mlit (9 files)

**Files:**
- Modify: `services/backend/lib/mlit/src/lib.rs` (already excellent — verify)
- Modify: `services/backend/lib/mlit/src/config.rs`
- Modify: `services/backend/lib/mlit/src/error.rs`
- Modify: `services/backend/lib/mlit/src/types.rs`
- Modify: `services/backend/lib/mlit/src/retry.rs`
- Modify: `services/backend/lib/mlit/src/reinfolib.rs`
- Modify: `services/backend/lib/mlit/src/jshis.rs`
- Modify: `services/backend/lib/mlit/src/estat.rs`
- Modify: `services/backend/lib/mlit/src/ksj.rs`

- [ ] **Step 1:** Read all 9 files
- [ ] **Step 2:** For `config.rs`, `error.rs`: add `//!` module docs. Document `MlitError` variants with `///` explaining each error case
- [ ] **Step 3:** For `reinfolib.rs`: add `//!` module doc explaining the Reinfolib API, authentication, and rate limits. Add `///` to all public functions with `# Errors` sections
- [ ] **Step 4:** For `estat.rs`, `ksj.rs`: add `//!` module docs explaining each government API's purpose and data types
- [ ] **Step 5:** For `retry.rs`: add `///` to the shared retry function with `# Examples` (using `no_run` since it requires network)
- [ ] **Step 6:** For `jshis.rs`: verify existing docs (EXCELLENT per audit), add intra-doc links
- [ ] **Step 7:** Run `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p terrasight-mlit && cargo test --doc -p terrasight-mlit`
- [ ] **Step 8:** Commit: `docs(mlit): add comprehensive rustdoc to terrasight-mlit`

---

## Task 4: terrasight-api domain layer (14 files)

**Files:**
- Modify: `services/backend/src/domain.rs`
- Modify: `services/backend/src/domain/entity.rs`
- Modify: `services/backend/src/domain/value_object.rs`
- Modify: `services/backend/src/domain/error.rs`
- Modify: `services/backend/src/domain/constants.rs`
- Modify: `services/backend/src/domain/repository.rs`
- Modify: `services/backend/src/domain/repository/mock.rs`
- Modify: `services/backend/src/domain/reinfolib.rs`
- Modify: `services/backend/src/domain/appraisal.rs`
- Modify: `services/backend/src/domain/municipality.rs`
- Modify: `services/backend/src/domain/transaction.rs`

- [ ] **Step 1:** Read all domain files
- [ ] **Step 2:** Add `//!` to `domain.rs`:
  - Explain Clean Architecture domain layer: pure business logic, no external deps
  - List sub-modules with one-line descriptions
  - Note: depends only on `std`, `serde`, `thiserror`, `chrono`, `async-trait`
- [ ] **Step 3:** In `entity.rs`, add `///` to every struct and every field. The `nonempty_string_type!` macro needs `///` explaining the pattern. Each entity needs docs explaining its business meaning (not just "A land price record" but "Represents a single real estate transaction record with geospatial coordinates and pricing data")
- [ ] **Step 4:** In `value_object.rs`, document every newtype's invariants (valid ranges, parsing rules). `BBox`, `PrefCode`, `Year`, `ZoomLevel` etc. need `# Examples` showing construction and validation failures
- [ ] **Step 5:** In `error.rs`, document every `DomainError` variant with `///` explaining when it occurs and which layer produces it
- [ ] **Step 6:** In `constants.rs`, add `///` to every constant. Group with `//` section separators. Explain the business meaning (e.g., `OPPORTUNITY_TLS_CONCURRENCY` — why 4?)
- [ ] **Step 7:** In `repository.rs`, add `//!` explaining the repository trait pattern. Each trait method needs `///` documenting its contract: what it accepts, what it returns, and under what conditions it errors. Add `# Errors` sections
- [ ] **Step 8:** In `appraisal.rs`, `municipality.rs`, `transaction.rs`, add `//!` module docs and `///` on all structs/fields
- [ ] **Step 9:** Run `cargo test --doc -p terrasight-api`
- [ ] **Step 10:** Commit: `docs(api/domain): add comprehensive rustdoc to domain layer`

---

## Task 5: terrasight-api handler layer (35 files)

**Files:**
- Modify: `services/backend/src/handler.rs`
- Modify: `services/backend/src/handler/error.rs`
- Modify: All `services/backend/src/handler/*.rs` (10 endpoint handlers)
- Modify: `services/backend/src/handler/request.rs` + all `request/*.rs` (8 files)
- Modify: `services/backend/src/handler/response.rs` + all `response/*.rs` (12 files)

- [ ] **Step 1:** Read `handler.rs`, `handler/error.rs`, `handler/request.rs`, `handler/response.rs`
- [ ] **Step 2:** Add `//!` to `handler.rs`:
  - Explain: HTTP handler layer — request validation, response formatting, error mapping
  - List endpoint groups with routes (e.g., "area_data — `GET /api/v1/area-data`")
  - Note: only layer with `axum` dependency
- [ ] **Step 3:** Add `//!` to `handler/request.rs`:
  - Explain: Axum query/path extractors with Serde deserialization and domain type conversion
  - Note: `into_domain()` / `into_filters()` pattern for crossing handler→usecase boundary
- [ ] **Step 4:** Add `//!` to `handler/response.rs`:
  - Explain: Response DTOs with `Serialize` for JSON output. `Dto` suffix convention
- [ ] **Step 5:** For each endpoint handler file (area_data, area_stats, land_price, opportunities, score, stats, trend, appraisals, municipalities, transactions, transaction_summary, health):
  - Add `//!` with: endpoint route, HTTP method, query parameters, response format
  - Add `///` to the handler function with `# Errors` listing possible error responses
- [ ] **Step 6:** For each `request/*.rs` file:
  - Add `//!` explaining which endpoint this request serves
  - Add `///` to query/path structs and their fields with validation rules
  - Document `default_*` functions explaining the business rationale for defaults
- [ ] **Step 7:** For each `response/*.rs` file:
  - Add `//!` explaining the response shape
  - Add `///` to all struct fields, especially computed fields
- [ ] **Step 8:** Run `cargo test --doc -p terrasight-api`
- [ ] **Step 9:** Commit: `docs(api/handler): add comprehensive rustdoc to handler layer`

---

## Task 6: terrasight-api infra + usecase layers (28 files)

**Files:**
- Modify: `services/backend/src/infra.rs`
- Modify: All `services/backend/src/infra/pg_*.rs` (10 repo files)
- Modify: `services/backend/src/infra/query_helpers.rs`
- Modify: `services/backend/src/infra/row_types.rs`
- Modify: `services/backend/src/infra/geo_convert.rs`
- Modify: `services/backend/src/infra/map_db_err.rs`
- Modify: `services/backend/src/infra/opportunities_cache.rs`
- Modify: `services/backend/src/infra/reinfolib_mock.rs`
- Modify: `services/backend/src/usecase.rs`
- Modify: All `services/backend/src/usecase/*.rs` (12 usecase files)
- Modify: `services/backend/src/lib.rs`
- Modify: `services/backend/src/app_state.rs`
- Modify: `services/backend/src/config.rs`
- Modify: `services/backend/src/logging.rs`

- [ ] **Step 1:** Read `infra.rs`, `usecase.rs`, `lib.rs`, `app_state.rs`, `config.rs`
- [ ] **Step 2:** Add `//!` to `infra.rs`:
  - Explain: PostgreSQL implementations of domain repository traits
  - Note: uses `run_query` helper for timeout + error mapping, PostGIS spatial queries
  - List sub-modules with one-line descriptions
- [ ] **Step 3:** For each `pg_*.rs`:
  - Add `//!` explaining: which repository trait it implements, key SQL patterns, PostGIS functions used
  - Add `///` to the struct (e.g., `PgAreaRepository`) and `new()` constructor
  - Add `///` with `# Errors` to each trait method implementation
- [ ] **Step 4:** In `query_helpers.rs`:
  - Add `# Examples` to `run_query` (using `no_run` for DB)
  - Add `# Examples` to `apply_limit` with actual values
- [ ] **Step 5:** In `row_types.rs`, `geo_convert.rs`:
  - Add `//!` module docs
  - Document each `FromRow` struct explaining the SQL it maps from
  - Document `Into<GeoFeature>` impls
- [ ] **Step 6:** Add `//!` to `usecase.rs`:
  - Explain: Business logic orchestration layer. Each usecase is a struct with `execute()`.
  - Note: depends only on domain traits (repository), injected via `AppState`
- [ ] **Step 7:** For each `usecase/*.rs`:
  - Add `//!` explaining business logic: what data it combines, what rules it applies
  - Add `///` to the usecase struct and `execute()` with `# Errors`
- [ ] **Step 8:** Update `lib.rs` crate-level docs:
  - Add architectural overview (handler → usecase → domain ← infra)
  - List all API endpoints with routes
  - Add Quick Start section for running the server
- [ ] **Step 9:** In `app_state.rs`:
  - Add `//!` explaining DI container pattern (Axum `FromRef`)
  - Document each field explaining which usecase it provides
- [ ] **Step 10:** In `config.rs`:
  - Add `///` to every config field explaining the env var name and default
- [ ] **Step 11:** Run `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p terrasight-api && cargo test --doc -p terrasight-api`
- [ ] **Step 12:** Commit: `docs(api): add comprehensive rustdoc to infra, usecase, and core modules`

---

## Task 7: terrasight-wasm (8 files)

**Files:**
- Modify: `services/wasm/src/lib.rs` (already OUTSTANDING — minor polish)
- Modify: `services/wasm/src/bbox.rs`
- Modify: `services/wasm/src/constants.rs`
- Modify: `services/wasm/src/error.rs`
- Modify: `services/wasm/src/fgb_reader.rs`
- Modify: `services/wasm/src/spatial_index.rs`
- Modify: `services/wasm/src/stats.rs`
- Modify: `services/wasm/src/tls.rs`

- [ ] **Step 1:** Read all 8 files
- [ ] **Step 2:** Add `//!` to files missing it: `bbox.rs`, `error.rs`
- [ ] **Step 3:** In `error.rs`, add `///` to each `WasmError` variant explaining when it occurs
- [ ] **Step 4:** In `bbox.rs`, add `///` to all methods with `# Examples` for `BBox::new` validation
- [ ] **Step 5:** In `stats.rs`, `spatial_index.rs`, `fgb_reader.rs`, `tls.rs`: verify existing docs, add any missing `///` on pub(crate) helper functions, add intra-doc links
- [ ] **Step 6:** Run `cd services/wasm && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps && cargo test --doc`
- [ ] **Step 7:** Commit: `docs(wasm): polish rustdoc for terrasight-wasm`

---

## Task 8: Final Verification + Cargo.toml metadata

**Files:**
- Modify: `services/backend/Cargo.toml`
- Modify: `services/backend/lib/domain/Cargo.toml`
- Modify: `services/backend/lib/server/Cargo.toml`
- Modify: `services/backend/lib/geo/Cargo.toml`
- Modify: `services/backend/lib/mlit/Cargo.toml`
- Modify: `services/wasm/Cargo.toml`

- [ ] **Step 1:** Add C-METADATA fields to all Cargo.toml files:
  ```toml
  authors = ["sdn03"]
  repository = "https://github.com/sdn0303/terrasight"
  license = "MIT"
  keywords = [...]
  categories = [...]
  ```
- [ ] **Step 2:** Full workspace doc build with warnings-as-errors:
  ```bash
  cd services/backend && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace
  cd services/wasm && RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
  ```
- [ ] **Step 3:** Run all doc tests:
  ```bash
  cd services/backend && cargo test --doc --workspace
  cd services/wasm && cargo test --doc
  ```
- [ ] **Step 4:** Verify zero broken intra-doc links (the RUSTDOCFLAGS check above catches these)
- [ ] **Step 5:** Run full test suite to ensure docs didn't break anything:
  ```bash
  cd services/backend && cargo clippy --workspace -- -D warnings && cargo test --workspace
  cd services/wasm && cargo clippy -- -D warnings && cargo test
  ```
- [ ] **Step 6:** Commit: `docs: add Cargo.toml metadata and verify documentation quality`

---

## Subagent Delegation Guide

| Task | Agent | Model | Files | Parallel |
|------|-------|-------|-------|----------|
| 0: terrasight-domain | `rust-engineer` | sonnet | 8 | Yes (with 1) |
| 1: terrasight-geo | `rust-engineer` | sonnet | 5 | Yes (with 0) |
| 2: terrasight-server | `rust-engineer` | sonnet | 20 | Yes (with 3) |
| 3: terrasight-mlit | `rust-engineer` | sonnet | 9 | Yes (with 2) |
| 4: api/domain | `rust-engineer` | sonnet | 14 | After 0-3 |
| 5: api/handler | `rust-engineer` | sonnet | 35 | After 4 |
| 6: api/infra+usecase | `rust-engineer` | sonnet | 28 | After 4 |
| 7: terrasight-wasm | `rust-engineer` | sonnet | 8 | Yes (with 0-3) |
| 8: Final verification | manual | — | 6 | Last |

**Parallelization:**
- Wave 1: Tasks 0, 1, 2, 3, 7 (all independent lib crates + WASM)
- Wave 2: Tasks 4, 5, 6 (terrasight-api — depends on lib crate docs for intra-doc links)
- Wave 3: Task 8 (verification — must be last)

---

## Risk Register

| Risk | Mitigation |
|------|-----------|
| Doc tests fail due to missing imports | Use `# ` hidden lines for setup; test after each task |
| Intra-doc links break across crates | `RUSTDOCFLAGS="-D warnings"` catches broken links |
| Over-documentation (verbose, noisy) | Keep summaries to 1 line; details only when non-obvious |
| Subagent adds unwanted code changes | Prompt explicitly: "documentation only, no code changes" |

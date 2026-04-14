# Type Safety & Code Quality Improvement Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rust 型システムを最大限活用し、raw primitive (`f64`, `&str`, `String`) をモジュール境界から排除。レビューで検出された Critical 1 / Important 16 / Medium 7 / Suggestion 8 件を修正。

**Architecture:** 型安全性の改善は bottom-up で実施: 共有型定義 (terrasight-domain/geo) → infra/server 層の引数変更 → handler/usecase 層の適合。各タスクは独立してコンパイル可能な単位。

**Tech Stack:** Rust 1.94, terrasight-domain/geo/server/mlit/api/wasm

**Branch:** 既存 `feature/workspace-restructure` に追加 push

---

## Phase 1: 共有 BBox/Coord 型の導入 (最大インパクト)

### Task 1.1: `terrasight-geo` に `GeoCoord` と `GeoBBox` を作成

> **注意:** Backend domain には既に `BBox` と `Coord` が存在するが、`terrasight-geo` は `terrasight-domain` や Backend に依存できない（依存方向が逆）。geo クレート内で使う軽量な型を作る。

**Files:**
- Create: `services/backend/lib/geo/src/coord.rs`
- Modify: `services/backend/lib/geo/src/lib.rs`
- Modify: `services/backend/lib/geo/src/spatial.rs`

- [ ] **Step 1:** `coord.rs` を作成。`GeoCoord { lng: f64, lat: f64 }` と `GeoBBox { south: f64, west: f64, north: f64, east: f64 }` を定義。`GeoBBox` にはバリデーションなし（geo は pure math、バリデーションは domain 層の責務）。
```rust
/// Unvalidated geographic coordinate pair (longitude, latitude).
///
/// Unlike [`terrasight_domain`]'s validated `Coord`, this struct carries
/// no invariants — it is a simple data carrier for pure math functions.
#[derive(Debug, Clone, Copy)]
pub struct GeoCoord {
    pub lng: f64,
    pub lat: f64,
}

/// Unvalidated bounding box in WGS-84 decimal degrees.
///
/// Field order is `(south, west, north, east)` — the same order used by
/// `ST_MakeEnvelope(west, south, east, north)` when read as (min_y, min_x, max_y, max_x).
#[derive(Debug, Clone, Copy)]
pub struct GeoBBox {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}
```

- [ ] **Step 2:** `lib.rs` に `pub mod coord;` 追加
- [ ] **Step 3:** `spatial.rs` の `bbox_area_deg2` を `GeoBBox` 引数に変更:
```rust
// Before: pub fn bbox_area_deg2(south: f64, west: f64, north: f64, east: f64) -> f64
// After:
pub fn bbox_area_deg2(bbox: &GeoBBox) -> f64 {
    ((bbox.north - bbox.south) * (bbox.east - bbox.west)).abs()
}
```
- [ ] **Step 4:** `spatial.rs` の `point_to_polygon` を `GeoCoord` 引数に変更:
```rust
// Before: pub fn point_to_polygon(lng: f64, lat: f64) -> [[f64; 2]; 5]
// After:
pub fn point_to_polygon(coord: &GeoCoord) -> [[f64; 2]; 5]
```
- [ ] **Step 5:** 全呼び出し元を更新（`pg_area_repository.rs`, `pg_land_price_repository.rs`, `handler/response/layer.rs`）
- [ ] **Step 6:** `compute_feature_limit` の `zoom: u32` → `zoom: u8` に変更（Web Mercator 0-22）
- [ ] **Step 7:** `BUFFER_DEG` → `POINT_TO_POLYGON_BUFFER_DEG` にリネーム
- [ ] **Step 8:** `bbox_to_tiles` に `Vec::with_capacity` 追加
- [ ] **Step 9:** ビルド検証: `cargo clippy --workspace -- -D warnings && cargo test --workspace`
- [ ] **Step 10:** コミット

### Task 1.2: `terrasight-server` の `bind_bbox` / `bind_coord` を型安全化

**Files:**
- Modify: `services/backend/lib/server/src/db/spatial.rs`
- Modify: 全 `pg_*.rs` 呼び出し元

- [ ] **Step 1:** `bind_bbox` を `GeoBBox` 引数に変更:
```rust
// Before: pub fn bind_bbox(query, west: f64, south: f64, east: f64, north: f64)
// After:
pub fn bind_bbox<'q, O>(query: ..., bbox: &GeoBBox) -> ... {
    query.bind(bbox.west).bind(bbox.south).bind(bbox.east).bind(bbox.north)
}
```
- [ ] **Step 2:** `bind_coord` を `GeoCoord` 引数に変更:
```rust
pub fn bind_coord<'q, O>(query: ..., coord: &GeoCoord) -> ...
```
- [ ] **Step 3:** `terrasight-server` の `Cargo.toml` に `terrasight-geo = { path = "../geo" }` 追加
- [ ] **Step 4:** 全呼び出し元更新（`pg_area_repository`, `pg_stats_repository`, `pg_tls_repository`, `pg_trend_repository`, `pg_land_price_repository`）。Backend domain の `BBox` から `GeoBBox` への変換は `GeoBBox { south: bbox.south(), west: bbox.west(), north: bbox.north(), east: bbox.east() }` で行う。
- [ ] **Step 5:** ビルド検証
- [ ] **Step 6:** コミット

---

## Phase 2: Domain Entity の型安全化

### Task 2.1: 新ドメインエンティティの raw String → newtype 変換

**Files:**
- Modify: `services/backend/src/domain/appraisal.rs`
- Modify: `services/backend/src/domain/municipality.rs`
- Modify: `services/backend/src/domain/transaction.rs`
- Modify: `services/backend/src/infra/pg_appraisal_repository.rs`
- Modify: `services/backend/src/infra/pg_municipality_repository.rs`
- Modify: `services/backend/src/infra/pg_transaction_repository.rs`
- Modify: `services/backend/src/handler/response/appraisal.rs`
- Modify: `services/backend/src/handler/response/municipality.rs`
- Modify: `services/backend/src/handler/response/transaction.rs`

- [ ] **Step 1:** `appraisal.rs` の `AppraisalDetail`:
  - `city_code: String` → `city_code: CityCode`
  - `city_name: String` → `city_name: AreaName`
  - `address: String` → `address: Address`
  - `zone_code: Option<String>` → `zone_code: Option<ZoneCode>`
- [ ] **Step 2:** `municipality.rs` の `Municipality`:
  - `city_code: String` → `city_code: CityCode`
  - `pref_code: String` → `pref_code: PrefCode`
  - `city_name: String` → `city_name: AreaName`
- [ ] **Step 3:** `transaction.rs` の `TransactionSummary` と `TransactionDetail`:
  - `city_code: String` → `city_code: CityCode`
- [ ] **Step 4:** 各 infra `pg_*.rs` の `From<Row>` impl を更新。DB から取得した `String` を newtype constructor で変換。infra 層なのでここで `parse().unwrap_or_default()` ではなく `parse().map_err(|_| DomainError::Validation(...))` とするか、信頼できる DB 値なので `.expect("INVARIANT: DB stores valid codes")` を使う。
- [ ] **Step 5:** 各 response DTO の `From<Entity>` impl を更新。`CityCode` → `.as_str().to_string()` 等
- [ ] **Step 6:** ビルド検証
- [ ] **Step 7:** コミット

### Task 2.2: Repository trait の `&str` → newtype 変更

**Files:**
- Modify: `services/backend/src/domain/repository.rs`
- Modify: `services/backend/src/domain/repository/mock.rs`
- Modify: `services/backend/src/infra/pg_transaction_repository.rs`
- Modify: `services/backend/src/infra/pg_appraisal_repository.rs`
- Modify: `services/backend/src/infra/pg_municipality_repository.rs`
- Modify: `services/backend/src/usecase/get_transactions.rs`
- Modify: `services/backend/src/usecase/get_appraisals.rs`
- Modify: `services/backend/src/usecase/get_municipalities.rs`
- Modify: `services/backend/src/handler/transactions.rs`
- Modify: `services/backend/src/handler/appraisals.rs`
- Modify: `services/backend/src/handler/municipalities.rs`

- [ ] **Step 1:** `TransactionRepository::find_transactions` の `city_code: &str` → `city_code: &CityCode`
- [ ] **Step 2:** `TransactionRepository::find_summary` の `pref_code: &str` → `pref_code: &PrefCode`
- [ ] **Step 3:** `AppraisalRepository::find_appraisals` の `pref_code: &str`, `city_code: Option<&str>` → `&PrefCode`, `Option<&CityCode>`
- [ ] **Step 4:** `MunicipalityRepository::find_municipalities` の `pref_code: &str` → `&PrefCode`
- [ ] **Step 5:** 各 usecase の `execute()` 引数を対応するように更新
- [ ] **Step 6:** 各 handler から usecase への呼び出しで `.as_str()` 除去（直接 newtype を渡す）
- [ ] **Step 7:** mock repository の引数も更新
- [ ] **Step 8:** ビルド検証
- [ ] **Step 9:** コミット

### Task 2.3: `OpportunitiesFilters.cities` → `Vec<CityCode>` + `GeoJsonType` enum

**Files:**
- Modify: `services/backend/src/domain/value_object.rs`
- Modify: `services/backend/src/domain/entity.rs`
- Modify: `services/backend/src/handler/request/opportunities.rs`
- Modify: `services/backend/src/infra/pg_area_repository.rs` (GeoJsonGeometry construction)

- [ ] **Step 1:** `OpportunitiesFilters.cities: Vec<String>` → `Vec<CityCode>`。handler の `into_filters()` で各 city code を `CityCode::new()` でバリデーション
- [ ] **Step 2:** `entity.rs` に `GeoJsonType` enum を作成:
```rust
#[derive(Debug, Clone, Serialize)]
pub enum GeoJsonType {
    Point,
    Polygon,
    MultiPolygon,
    LineString,
}
impl GeoJsonType {
    pub fn as_str(&self) -> &'static str { ... }
}
```
- [ ] **Step 3:** `GeoJsonGeometry.r#type: String` → `GeoJsonType`
- [ ] **Step 4:** infra の `to_geo_feature` で DB の `geometry_type` 文字列を `GeoJsonType` に変換
- [ ] **Step 5:** ビルド検証
- [ ] **Step 6:** コミット

---

## Phase 3: Scoring 関数シグネチャ改善

### Task 3.1: `compute_tls` を struct 引数に変更

**Files:**
- Modify: `services/backend/lib/domain/src/scoring/tls.rs`
- Modify: `services/backend/src/usecase/compute_tls.rs`

- [ ] **Step 1:** `AxisScores` struct を定義:
```rust
/// Normalized axis scores (0-100 scale) for the 5-axis TLS formula.
pub struct AxisScores {
    pub s1_disaster: f64,
    pub s2_terrain: f64,
    pub s3_livability: f64,
    pub s4_future: f64,
    pub s5_profitability: f64,
}
```
- [ ] **Step 2:** `compute_tls(s1, s2, s3, s4, s5, preset)` → `compute_tls(scores: &AxisScores, preset: WeightPreset)`
- [ ] **Step 3:** `compute_cross_analysis` も同様に struct 引数化
- [ ] **Step 4:** 呼び出し元 (`compute_tls.rs`) を更新
- [ ] **Step 5:** テスト更新
- [ ] **Step 6:** ビルド検証
- [ ] **Step 7:** コミット

### Task 3.2: `#[must_use]` を全 pure function に追加

**Files:**
- Modify: `services/backend/lib/domain/src/scoring/tls.rs`
- Modify: `services/backend/lib/domain/src/scoring/axis.rs`
- Modify: `services/backend/lib/domain/src/scoring/sub_scores.rs`
- Modify: `services/backend/lib/geo/src/spatial.rs`
- Modify: `services/backend/lib/geo/src/finance.rs`
- Modify: `services/backend/lib/geo/src/rounding.rs`
- Modify: `services/backend/lib/geo/src/tile.rs`

- [ ] **Step 1:** 全ての `pub fn` で戻り値が主目的の pure function に `#[must_use]` を追加
- [ ] **Step 2:** ビルド検証（clippy が未使用戻り値を検出）
- [ ] **Step 3:** コミット

---

## Phase 4: mlit クレート改善

### Task 4.1: Reinfolib エンドポイント定数化 + serde round-trip 除去

**Files:**
- Modify: `services/backend/lib/mlit/src/reinfolib.rs`

- [ ] **Step 1:** エンドポイントコード定数を追加:
```rust
const ENDPOINT_TRANSACTION_PRICES: &str = "XPT001";
const ENDPOINT_LAND_PRICES: &str = "XPT002";
const ENDPOINT_INSTITUTIONS: &str = "XIT001";
// ... etc
```
- [ ] **Step 2:** `fetch_tile_features` の `serde_json::to_value(&feature)` round-trip を除去。`GeoJsonResponse.features` を `Vec<serde_json::Value>` に変更するか、`GeoJsonFeature` をそのまま返す
- [ ] **Step 3:** `request_with_retry` の引数 `&[(String, String)]` → `&[(&str, String)]` に変更（key は static str）
- [ ] **Step 4:** ビルド検証
- [ ] **Step 5:** コミット

### Task 4.2: `JshisClient::new` の引数を `&MlitConfig` に統一

**Files:**
- Modify: `services/backend/lib/mlit/src/jshis.rs`
- Modify: `services/backend/src/app_state.rs`

- [ ] **Step 1:** `JshisClient::new(timeout_secs: u64)` → `JshisClient::new(config: &MlitConfig)` に変更
- [ ] **Step 2:** `app_state.rs` の `JshisClient` 構築を更新
- [ ] **Step 3:** ビルド検証
- [ ] **Step 4:** コミット

---

## Phase 5: 安全性と細かい改善

### Task 5.1: production `.unwrap()` 除去 + `as` キャスト安全化

**Files:**
- Modify: `services/backend/src/handler/request/bbox.rs`
- Modify: `services/backend/src/handler/request/opportunities.rs`
- Modify: `services/wasm/src/lib.rs`
- Modify: `services/wasm/src/spatial_index.rs` (features.len() as u32)

- [ ] **Step 1:** `bbox.rs:87` と `opportunities.rs:86` の `.unwrap()` → `.expect("INVARIANT: WeightPreset::FromStr is infallible")`
- [ ] **Step 2:** WASM `lib.rs:264` の `features.len() as u32` → `u32::try_from(features.len()).expect("INVARIANT: feature count fits in u32")`
- [ ] **Step 3:** `spatial_index.rs` の同様のキャストも修正
- [ ] **Step 4:** `rate_limit.rs` の `.expect()` に `INVARIANT:` prefix 追加
- [ ] **Step 5:** ビルド検証
- [ ] **Step 6:** コミット

### Task 5.2: DRY 改善 + minor fixes

**Files:**
- Modify: `services/backend/src/app_state.rs` (PgTransactionRepository 共有)
- Modify: `services/backend/src/usecase/compute_tls.rs` (compute_price_cagr precondition doc)
- Modify: `services/backend/src/domain/value_object.rs` (OpportunitiesCacheKey 順序整合)

- [ ] **Step 1:** `app_state.rs` で `PgTransactionRepository` を1つの `Arc` で共有:
```rust
let tx_repo = Arc::new(PgTransactionRepository::new(pool.clone()));
// transaction_summary: Arc::new(GetTransactionSummaryUsecase::new(tx_repo.clone())),
// transactions: Arc::new(GetTransactionsUsecase::new(tx_repo)),
```
- [ ] **Step 2:** `compute_price_cagr` にソート前提条件のドキュメントと `debug_assert!` 追加
- [ ] **Step 3:** ビルド検証
- [ ] **Step 4:** コミット

---

## Phase 6: 最終検証

### Task 6.1: 全体ビルド + テスト + doc

- [ ] **Step 1:** Backend: `cargo fmt --all && cargo clippy --workspace -- -D warnings && cargo test --workspace`
- [ ] **Step 2:** WASM: `cargo fmt --all && cargo clippy -- -D warnings && cargo test`
- [ ] **Step 3:** Doc: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --workspace`
- [ ] **Step 4:** 旧 primitive 残存チェック:
```bash
# bind_bbox/bind_coord が raw f64 を受けていないか
grep -rn 'fn bind_bbox.*f64' services/backend/lib/server/src/
grep -rn 'fn bind_coord.*f64' services/backend/lib/server/src/
# bbox_area_deg2 が raw f64 を受けていないか
grep -rn 'fn bbox_area_deg2.*south' services/backend/lib/geo/src/
```
- [ ] **Step 5:** コミット + push

---

## Subagent Delegation Guide

| Task | Agent | Model | Files | 並行可否 |
|------|-------|-------|-------|----------|
| 1.1 geo 型導入 | `rust-engineer` | sonnet | 8 | Yes (with 2.1) |
| 1.2 server bind 改善 | `rust-engineer` | sonnet | 12 | After 1.1 |
| 2.1 entity raw String 除去 | `rust-engineer` | sonnet | 9 | Yes (with 1.1) |
| 2.2 repository trait 型安全化 | `rust-engineer` | sonnet | 11 | After 2.1 |
| 2.3 cities Vec + GeoJsonType | `rust-engineer` | sonnet | 4 | After 2.2 |
| 3.1 compute_tls struct 化 | `rust-engineer` | sonnet | 2 | After 1.1 |
| 3.2 must_use 追加 | `rust-engineer` | haiku | 7 | After 3.1 |
| 4.1 reinfolib 改善 | `rust-engineer` | sonnet | 1 | Independent |
| 4.2 jshis config 統一 | `rust-engineer` | haiku | 2 | After 4.1 |
| 5.1 unwrap/cast 安全化 | `rust-engineer` | haiku | 4 | Independent |
| 5.2 DRY minor fixes | `rust-engineer` | haiku | 3 | Independent |
| 6.1 最終検証 | manual | — | — | Last |

---

## Risk Register

| Risk | Mitigation |
|------|-----------|
| `GeoBBox` と domain `BBox` の二重定義 | `GeoBBox` は unvalidated math struct、`BBox` は validated domain type。`From<&BBox> for GeoBBox` impl で変換 |
| infra で DB 値を newtype 変換する際の panic | `expect("INVARIANT: DB stores valid X")` を使用。DB スキーマが保証する不変条件 |
| `request_with_retry` の引数変更が mlit 全体に波及 | key を `&str` に、value だけ `String` に。影響は reinfolib + jshis の2ファイル |
| scoring struct 化で WASM 側に影響 | WASM は独自の `compute_tls` を持つため影響なし |

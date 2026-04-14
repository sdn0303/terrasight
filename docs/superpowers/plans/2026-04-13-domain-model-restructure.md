# Domain Model Restructure Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** `domain/entity.rs` (660行) と `domain/value_object.rs` (978行) を責務ごとのファイルに分割し、`domain/model/` 配下に再配置。全 import パスを `crate::domain::model::*` に統一。import はクレート毎にグルーピング。

**Architecture:** `model.rs` が全型を re-export するため、外部コードは `use crate::domain::model::{BBox, GeoFeature, ...}` の単一パスで全型にアクセス可能。旧パス (`crate::domain::entity::*`, `crate::domain::value_object::*`) は廃止。

**Tech Stack:** Rust 1.94 / terrasight-api

**Branch:** 既存 `feature/workspace-restructure` に追加

---

## Before → After 構造

### Before
```
domain/
├── domain.rs
├── entity.rs              ← 660行 (27型混在)
├── value_object.rs        ← 978行 (19型混在)
├── error.rs
├── constants.rs
├── appraisal.rs           ← 1型
├── municipality.rs        ← 1型
├── transaction.rs         ← 2型
├── reinfolib.rs
├── repository.rs
└── repository/            ← 10 trait files + mock.rs
```

### After
```
domain/
├── domain.rs              ← module index
├── model.rs               ← model module index + re-export
├── model/
│   ├── primitives.rs      ← 汎用 validated newtypes (BBox, Coord, PrefCode, CityCode, Year, ZoomLevel, Meters, Percent, etc.)
│   ├── geo.rs             ← GeoJSON types (GeoFeature, GeoJsonType, GeoJsonGeometry, LayerResult, LayerType)
│   ├── price.rs           ← 地価関連 (PricePerSqm, PriceRecord, LandPriceStats re-export)
│   ├── tls.rs             ← TLS sub-score types (SchoolStats, MedicalStats, ZScoreResult, TlsScore, RiskLevel)
│   ├── opportunity.rs     ← 投資機会 (OpportunityRecord, Opportunity, StationHint, Cached*, Filters, CacheKey, Signal, Limit, Offset)
│   ├── trend.rs           ← 価格トレンド (TrendPoint, TrendLocation, TrendAnalysis, TrendDirection, YearsLookback)
│   ├── area.rs            ← エリア統計 (AreaStats, FacilityStats, AdminAreaStats, AreaCode, AreaCodeLevel, AreaName)
│   ├── appraisal.rs       ← 鑑定評価 (AppraisalDetail)
│   ├── municipality.rs    ← 市区町村 (Municipality)
│   ├── transaction.rs     ← 取引 (TransactionSummary, TransactionDetail)
│   └── health.rs          ← ヘルスチェック (HealthStatus)
├── error.rs               ← 変更なし
├── constants.rs           ← 変更なし
├── reinfolib.rs           ← import パスのみ更新
├── repository.rs          ← 変更なし
└── repository/            ← import パスのみ更新
```

## 型の移動マッピング

### primitives.rs (旧 value_object.rs + entity.rs の汎用 newtypes)
| 型 | 旧ファイル |
|---|---|
| `BBox` | value_object.rs |
| `Coord` | value_object.rs |
| `Year` | value_object.rs |
| `ZoomLevel` | value_object.rs |
| `PrefCode` | value_object.rs |
| `CityCode` | value_object.rs |
| `Address` | entity.rs |
| `ZoneCode` | entity.rs |
| `Meters` | entity.rs |
| `Percent` | entity.rs |
| `RecordCount` | entity.rs |
| `PricePerSqm` | entity.rs |
| `BuildingCoverageRatio` | entity.rs |
| `FloorAreaRatio` | entity.rs |

### geo.rs (GeoJSON 関連)
| 型 | 旧ファイル |
|---|---|
| `GeoFeature` | entity.rs |
| `GeoJsonType` | entity.rs |
| `GeoJsonGeometry` | entity.rs |
| `LayerResult` | entity.rs |
| `LayerType` | value_object.rs |

### price.rs (地価)
| 型 | 旧ファイル |
|---|---|
| `PriceRecord` | entity.rs |
| `LandPriceStats` | entity.rs (re-export from terrasight-domain) |

### tls.rs (TLS sub-score)
| 型 | 旧ファイル |
|---|---|
| `SchoolStats` | entity.rs |
| `MedicalStats` | entity.rs |
| `ZScoreResult` | entity.rs |
| `TlsScore` | value_object.rs |
| `RiskLevel` | value_object.rs |

### opportunity.rs (投資機会)
| 型 | 旧ファイル |
|---|---|
| `OpportunityRecord` | entity.rs |
| `Opportunity` | entity.rs |
| `StationHint` | entity.rs |
| `CachedOpportunitiesResponse` | entity.rs |
| `OpportunitySignal` | value_object.rs |
| `OpportunityLimit` | value_object.rs |
| `OpportunityOffset` | value_object.rs |
| `OpportunitiesFilters` | value_object.rs |
| `OpportunitiesCacheKey` | value_object.rs |

### trend.rs (トレンド)
| 型 | 旧ファイル |
|---|---|
| `TrendPoint` | entity.rs |
| `TrendLocation` | entity.rs |
| `TrendAnalysis` | value_object.rs |
| `TrendDirection` | value_object.rs |
| `YearsLookback` | value_object.rs |

### area.rs (エリア統計)
| 型 | 旧ファイル |
|---|---|
| `AreaStats` | entity.rs |
| `FacilityStats` | entity.rs |
| `AdminAreaStats` | entity.rs |
| `AreaCode` | value_object.rs |
| `AreaCodeLevel` | value_object.rs |
| `AreaName` | entity.rs |
| `RiskStats` | entity.rs (re-export from terrasight-domain) |

### health.rs
| 型 | 旧ファイル |
|---|---|
| `HealthStatus` | entity.rs |

### appraisal.rs, municipality.rs, transaction.rs
既存ファイルをそのまま `model/` に移動。import パスのみ更新。

---

## Import パス変更ルール

### 旧 → 新 パスマッピング
```
crate::domain::entity::*        → crate::domain::model::*
crate::domain::value_object::*  → crate::domain::model::*
crate::domain::appraisal::*     → crate::domain::model::*
crate::domain::municipality::*  → crate::domain::model::*
crate::domain::transaction::*   → crate::domain::model::*
```

`model.rs` が全型を flat に re-export するため、consumer は `use crate::domain::model::{BBox, GeoFeature, PrefCode}` の一行で全てアクセス可能。

### Import グルーピング規則
```rust
// 1. std library
use std::collections::HashMap;
use std::sync::Arc;

// 2. External crates (alphabetical)
use async_trait::async_trait;
use axum::extract::{Query, State};
use serde::Serialize;

// 3. Workspace crates
use terrasight_domain::scoring::tls::WeightPreset;
use terrasight_geo::GeoBBox;
use terrasight_server::db::spatial::bind_bbox;

// 4. Crate-internal (crate::)
use crate::domain::constants::*;
use crate::domain::error::DomainError;
use crate::domain::model::{BBox, Coord, PrefCode};
use crate::domain::repository::LayerRepository;
```

各グループは空行で区切る。グループ内はアルファベット順。

---

## Task 分割

### Task 0: model/ ディレクトリ作成 + 型の分割配置

**作業:** `entity.rs` と `value_object.rs` の内容を 11 ファイルに分割。`model.rs` で re-export。

**ファイル作成順 (依存順):**
1. `model/primitives.rs` — 他の model ファイルが依存する基本型
2. `model/geo.rs` — primitives に依存
3. `model/price.rs` — primitives に依存
4. `model/tls.rs` — primitives に依存
5. `model/trend.rs` — primitives に依存
6. `model/area.rs` — primitives に依存
7. `model/opportunity.rs` — primitives, geo, tls に依存
8. `model/health.rs` — 依存なし
9. `model/appraisal.rs` — primitives に依存 (既存移動)
10. `model/municipality.rs` — primitives に依存 (既存移動)
11. `model/transaction.rs` — primitives に依存 (既存移動)
12. `model.rs` — 全 re-export

**注意:** 各ファイル内で必要な `use crate::domain::error::DomainError` や `use crate::domain::constants::*` は直接 import する。model ファイル間の依存は `use super::primitives::*` ではなく `use crate::domain::model::*` を使う（循環防止のため re-export 経由）。

実際には model.rs の re-export は各サブモジュールの pub items を glob で再公開するので、model 内部では `use super::primitives::PrefCode` 等で参照可能。

- [ ] **Step 1:** ディレクトリ作成
- [ ] **Step 2:** 各ファイルを作成（型定義 + impl + tests を移動）
- [ ] **Step 3:** `model.rs` を作成（mod 宣言 + pub use re-export）
- [ ] **Step 4:** 旧 `entity.rs` と `value_object.rs` を削除
- [ ] **Step 5:** 旧 `domain/appraisal.rs`, `municipality.rs`, `transaction.rs` を `model/` に移動
- [ ] **Step 6:** `domain.rs` の mod 宣言を更新（entity, value_object 削除、model 追加）
- [ ] **Step 7:** ビルド確認（domain 内のみ）
- [ ] **Step 8:** コミット

### Task 1: domain 内部 import 更新

**作業:** `domain/` 内のファイル（error.rs, constants.rs, reinfolib.rs, repository/*.rs）の import パスを更新。

- [ ] **Step 1:** `domain/error.rs` — import 不要（DomainError は他から参照される側）
- [ ] **Step 2:** `domain/constants.rs` — import 確認（model 型を参照していれば更新）
- [ ] **Step 3:** `domain/reinfolib.rs` — `crate::domain::entity::*` → `crate::domain::model::*`
- [ ] **Step 4:** `domain/repository/*.rs` (10ファイル) — 全 entity/value_object import を model に変更
- [ ] **Step 5:** `domain/repository/mock.rs` — 同上
- [ ] **Step 6:** ビルド確認
- [ ] **Step 7:** コミット

### Task 2: handler 層 import 更新

**作業:** `handler/` 配下全ファイル (~20ファイル) の import を更新。

- [ ] **Step 1:** `handler/error.rs`
- [ ] **Step 2:** `handler/request/*.rs` (8ファイル)
- [ ] **Step 3:** `handler/response/*.rs` (11ファイル)
- [ ] **Step 4:** `handler/*.rs` endpoint ファイル (trend.rs, transactions.rs 等)
- [ ] **Step 5:** 全ファイルで import をクレート毎にグルーピング
- [ ] **Step 6:** ビルド確認
- [ ] **Step 7:** コミット

### Task 3: infra 層 import 更新

**作業:** `infra/` 配下全ファイル (~15ファイル) の import を更新。

- [ ] **Step 1:** 全 `pg_*.rs` — `crate::domain::entity::*` と `crate::domain::value_object::*` を `crate::domain::model::*` に統合
- [ ] **Step 2:** `infra/query_helpers.rs`, `geo_convert.rs`, `map_db_err.rs`, `row_types.rs`, `opportunities_cache.rs`, `reinfolib_mock.rs`
- [ ] **Step 3:** 全ファイルで import をクレート毎にグルーピング
- [ ] **Step 4:** ビルド確認
- [ ] **Step 5:** コミット

### Task 4: usecase 層 import 更新

**作業:** `usecase/` 配下全ファイル (~13ファイル) の import を更新。

- [ ] **Step 1:** 全 usecase ファイル — `crate::domain::entity::*` と `crate::domain::value_object::*` を `crate::domain::model::*` に統合
- [ ] **Step 2:** 全ファイルで import をクレート毎にグルーピング
- [ ] **Step 3:** ビルド確認
- [ ] **Step 4:** コミット

### Task 5: app_state.rs + lib.rs + tests import 更新

**作業:** 残りのファイル

- [ ] **Step 1:** `app_state.rs`
- [ ] **Step 2:** `lib.rs`
- [ ] **Step 3:** `tests/api_integration.rs`
- [ ] **Step 4:** 全体ビルド + テスト + doc 検証
- [ ] **Step 5:** コミット

---

## Subagent Delegation Guide

| Task | Agent | Model | Max Files | 備考 |
|------|-------|-------|-----------|------|
| 0: model/ 分割 | `rust-engineer` | sonnet | 15 | 最大タスク — 型の移動 |
| 1: domain 内部 | `rust-engineer` | sonnet | 12 | repository mock 含む |
| 2: handler | `rust-engineer` | sonnet | 20 | request + response |
| 3: infra | `rust-engineer` | sonnet | 15 | pg_* 全ファイル |
| 4: usecase | `rust-engineer` | sonnet | 13 | |
| 5: core + verify | `rust-engineer` | sonnet | 3 | 最終検証 |

全タスク直列実行（各タスクが前タスクのコンパイル成功に依存）。

---

## Risk Register

| Risk | Mitigation |
|------|-----------|
| model 内の循環参照 | `opportunity.rs` が `geo.rs` の `GeoFeature` を参照 → `super::geo::GeoFeature` で解決。re-export は model.rs が一括管理 |
| 234 import 行の書き換え漏れ | `cargo clippy` が未使用 import と未解決パスを全検出 |
| test 内の import 漏れ | `cargo test --workspace` で全テスト通過を確認 |
| 旧パスを使う外部ファイル | `grep -rn 'domain::entity\|domain::value_object\|domain::appraisal\|domain::municipality\|domain::transaction' src/` で残存チェック |

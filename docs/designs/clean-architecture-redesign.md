# Clean Architecture 再設計計画書

> 既存 Rust Axum バックエンドを Clean Architecture 4層に厳密分離する設計ドキュメント

---

## Goal

**handler → usecase → domain ← infra** の依存方向を厳守し、各層の責務を明確化。
Domain 層にゼロ外部依存、Repository trait による DI でテスタビリティ確保。

---

## 1. 現状の問題点

| # | 問題 | 該当箇所 | 影響 |
|---|------|---------|------|
| 1 | **Domain 層が存在しない** | `models/` にエンティティ・DTO・API型が混在 | 責務不明瞭、変更波及 |
| 2 | **Repository trait なし** | `infra/repo_*.rs` が `PgPool` を直接受取るフリー関数 | テスト時モック不可 |
| 3 | **Handler にビジネスロジック漏洩** | `area_data.rs` のレイヤー分岐、`trend.rs` の CAGR 計算 | 単体テスト困難 |
| 4 | **Usecase 層が不完全** | `scoring.rs` のみ。他3エンドポイントはハンドラ直結 | 一貫性なし |
| 5 | **AppState が具象型保持** | `db: PgPool` 直保持 | DI 不可能 |
| 6 | **レスポンス型が Domain 型と兼用** | `responses.rs` が serde::Serialize のみ | 層間の結合度が高い |
| 7 | **エラー型が axum に依存** | `AppError` が `IntoResponse` 実装 | Domain 層で使えない |

---

## 2. 新ディレクトリ構造

```
services/backend/src/
├── main.rs                      # DI wiring + Axum Router 構築
├── config.rs                    # 環境変数構造体（変更なし）
├── logging.rs                   # 構造化ロギング（変更なし）
│
├── domain/                      # 🔴 純粋層: 外部依存ゼロ
│   ├── mod.rs
│   ├── entity.rs                # エンティティ（GeoFeature, PriceRecord, etc.）
│   ├── value_object.rs          # 値オブジェクト（BBox, Coord, LayerType, Score）
│   ├── repository.rs            # Repository trait 定義（async_trait）
│   └── error.rs                 # DomainError（フレームワーク非依存）
│
├── usecase/                     # 🟡 ビジネスロジック: Domain のみに依存
│   ├── mod.rs
│   ├── get_area_data.rs         # エリアデータ取得ユースケース
│   ├── compute_score.rs         # 投資スコア算出ユースケース
│   ├── get_stats.rs             # エリア統計集計ユースケース
│   ├── get_trend.rs             # 地価推移取得ユースケース
│   └── check_health.rs          # ヘルスチェックユースケース
│
├── infra/                       # 🟢 インフラ: Domain trait を実装
│   ├── mod.rs
│   ├── pg_area_repository.rs    # AreaRepository の PostGIS 実装
│   ├── pg_score_repository.rs   # ScoreRepository の PostGIS 実装
│   ├── pg_stats_repository.rs   # StatsRepository の PostGIS 実装
│   ├── pg_trend_repository.rs   # TrendRepository の PostGIS 実装
│   └── pg_health_repository.rs  # HealthRepository の PostGIS 実装
│
├── handler/                     # 🔵 HTTP層: Axum 固有コード
│   ├── mod.rs
│   ├── request.rs               # リクエストDTO（Axum Query 用 Deserialize）
│   ├── response.rs              # レスポンスDTO（Axum Json 用 Serialize）
│   ├── error.rs                 # AppError → HTTP レスポンス変換
│   ├── health.rs
│   ├── area_data.rs
│   ├── score.rs
│   ├── stats.rs
│   └── trend.rs
│
└── tests/                       # 統合テスト
    └── health_test.rs
```

---

## 3. Domain 層（外部依存ゼロ ※例外あり）

> **許容依存の例外宣言:** `serde_json::Value` は Domain 層で使用を許容する。
> 理由: GeoJSON の座標配列やスコア詳細は構造が動的であり、stdlib 型で表現すると
> オーバーエンジニアリングになる。`serde_json` は「データ表現ライブラリ」であり
> 「フレームワーク」ではないため、Domain の純粋性を実質的に損なわない。
> 禁止対象は `axum`, `sqlx`, `reqwest` 等のI/Oフレームワーク。

### 3.1 エンティティ — `domain/entity.rs`

```rust
/// GeoJSON Feature のドメイン表現。
/// PostGIS の ST_AsGeoJSON 出力と 1:1 対応。
#[derive(Debug, Clone)]
pub struct GeoFeature {
    pub geometry: GeoJsonGeometry,      // [lng, lat] RFC 7946
    pub properties: GeoJsonProperties,
}

/// GeoJSON geometry（Value で柔軟に保持）
#[derive(Debug, Clone)]
pub struct GeoJsonGeometry {
    pub r#type: String,                 // "Point", "MultiPolygon", etc.
    pub coordinates: serde_json::Value, // 座標配列
}

/// GeoJSON properties（レイヤーによって異なるため Value）
pub type GeoJsonProperties = serde_json::Value;

/// 地価レコード（スコアリング・トレンド計算用）
#[derive(Debug, Clone)]
pub struct PriceRecord {
    pub year: i32,
    pub price_per_sqm: i64,
    pub address: String,
    pub distance_m: f64,
}

/// トレンドデータポイント
#[derive(Debug, Clone)]
pub struct TrendPoint {
    pub year: i32,
    pub price_per_sqm: i64,
}

/// トレンド観測地点情報
#[derive(Debug, Clone)]
pub struct TrendLocation {
    pub address: String,
    pub distance_m: f64,
}

/// 地価統計（集計結果）
#[derive(Debug, Clone)]
pub struct LandPriceStats {
    pub avg_per_sqm: Option<f64>,
    pub median_per_sqm: Option<f64>,
    pub min_per_sqm: Option<i64>,
    pub max_per_sqm: Option<i64>,
    pub count: i64,
}

/// リスク統計
#[derive(Debug, Clone)]
pub struct RiskStats {
    pub flood_area_ratio: f64,
    pub steep_slope_area_ratio: f64,
    pub composite_risk: f64,
}

/// 施設統計
#[derive(Debug, Clone)]
pub struct FacilityStats {
    pub schools: i64,
    pub medical: i64,
}
```

### 3.2 値オブジェクト — `domain/value_object.rs`

```rust
use crate::domain::error::DomainError;

/// バウンディングボックス（不変条件: south < north, west < east, 各辺 ≤ 0.5°）
#[derive(Debug, Clone)]
pub struct BBox {
    south: f64,
    west: f64,
    north: f64,
    east: f64,
}

impl BBox {
    /// バリデーション付きコンストラクタ。不正値は生成不可能。
    pub fn new(south: f64, west: f64, north: f64, east: f64) -> Result<Self, DomainError> {
        if !(-90.0..=90.0).contains(&south) || !(-90.0..=90.0).contains(&north) {
            return Err(DomainError::InvalidCoordinate("latitude must be -90..90".into()));
        }
        if !(-180.0..=180.0).contains(&west) || !(-180.0..=180.0).contains(&east) {
            return Err(DomainError::InvalidCoordinate("longitude must be -180..180".into()));
        }
        if south >= north {
            return Err(DomainError::InvalidCoordinate("south must be less than north".into()));
        }
        if west >= east {
            return Err(DomainError::InvalidCoordinate("west must be less than east".into()));
        }
        if (north - south) > 0.5 || (east - west) > 0.5 {
            return Err(DomainError::BBoxTooLarge);
        }
        Ok(Self { south, west, north, east })
    }

    // Getter（不変条件が破壊されないようフィールドは非公開）
    pub fn south(&self) -> f64 { self.south }
    pub fn west(&self) -> f64 { self.west }
    pub fn north(&self) -> f64 { self.north }
    pub fn east(&self) -> f64 { self.east }
}

/// 座標（不変条件: lat -90..90, lng -180..180）
#[derive(Debug, Clone)]
pub struct Coord {
    lat: f64,
    lng: f64,
}

impl Coord {
    pub fn new(lat: f64, lng: f64) -> Result<Self, DomainError> {
        if !(-90.0..=90.0).contains(&lat) {
            return Err(DomainError::InvalidCoordinate("latitude must be -90..90".into()));
        }
        if !(-180.0..=180.0).contains(&lng) {
            return Err(DomainError::InvalidCoordinate("longitude must be -180..180".into()));
        }
        Ok(Self { lat, lng })
    }

    pub fn lat(&self) -> f64 { self.lat }
    pub fn lng(&self) -> f64 { self.lng }
}

/// 取得対象レイヤー種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LayerType {
    LandPrice,
    Zoning,
    Flood,
    SteepSlope,
    Schools,
    Medical,
}

impl LayerType {
    /// 文字列パース。不明なレイヤーは None。
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "landprice" => Some(Self::LandPrice),
            "zoning" => Some(Self::Zoning),
            "flood" => Some(Self::Flood),
            "steep_slope" => Some(Self::SteepSlope),
            "schools" => Some(Self::Schools),
            "medical" => Some(Self::Medical),
            _ => None,
        }
    }

    /// REST API 用キー文字列
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LandPrice => "landprice",
            Self::Zoning => "zoning",
            Self::Flood => "flood",
            Self::SteepSlope => "steep_slope",
            Self::Schools => "schools",
            Self::Medical => "medical",
        }
    }
}

/// 投資スコア（不変条件: 各コンポーネント 0..=25, 合計 0..=100）
#[derive(Debug, Clone)]
pub struct InvestmentScore {
    pub trend: ScoreComponent,
    pub risk: ScoreComponent,
    pub access: ScoreComponent,
    pub yield_potential: ScoreComponent,
}

impl InvestmentScore {
    pub fn total(&self) -> f64 {
        self.trend.value + self.risk.value + self.access.value + self.yield_potential.value
    }
}

#[derive(Debug, Clone)]
pub struct ScoreComponent {
    pub value: f64,       // 0.0..=25.0
    pub max: f64,         // 常に 25.0
    pub detail: serde_json::Value,
}

/// トレンド分析結果
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    pub location: TrendLocation,
    pub data: Vec<TrendPoint>,
    pub cagr: f64,
    pub direction: TrendDirection,
}

#[derive(Debug, Clone, Copy)]
pub enum TrendDirection {
    Up,
    Down,
}

impl TrendDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
        }
    }
}
```

`★ Insight ─────────────────────────────────────`
**値オブジェクトの「不変条件をコンストラクタで保証」パターン:**

- `BBox::new()` は `Result` を返し、不正な座標や大きすぎる範囲を**生成時に拒否**
- フィールドを `private` にし、getter のみ公開することで、**一度作られた `BBox` は常に有効**
- これにより、下流の usecase/infra では `bbox.validate()` の呼び出しが**不要になる**
- 現在の `BBoxParams.validate()` は「生成後に呼ぶ必要がある」ため呼び忘れリスクがある

これは DDD の **Value Object** パターンの核心：不変条件をコンパイル時（型）＋実行時（コンストラクタ）で強制する
`─────────────────────────────────────────────────`

### 3.3 Repository trait — `domain/repository.rs`

```rust
use async_trait::async_trait;
use std::collections::HashMap;

use crate::domain::entity::*;
use crate::domain::error::DomainError;
use crate::domain::value_object::*;

// ─── Area Data ───────────────────────────────────────
// レイヤーごとにメソッドを分割。
// 理由: 各テーブルのスキーマが異なり、将来的にレイヤー固有の戻り値型を返す拡張性を確保。

#[async_trait]
pub trait AreaRepository: Send + Sync {
    async fn find_land_prices(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError>;
    async fn find_zoning(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError>;
    async fn find_flood_risk(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError>;
    async fn find_steep_slope(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError>;
    async fn find_schools(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError>;
    async fn find_medical(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError>;
}

// ─── Score ───────────────────────────────────────────

#[async_trait]
pub trait ScoreRepository: Send + Sync {
    /// 最寄り地点の複数年度地価データ
    async fn find_nearest_prices(
        &self,
        coord: &Coord,
    ) -> Result<Vec<PriceRecord>, DomainError>;

    /// 半径 500m 内の洪水リスク重畳率 (0.0..1.0)
    async fn calc_flood_overlap(
        &self,
        coord: &Coord,
    ) -> Result<f64, DomainError>;

    /// 半径 500m 内に急傾斜地が存在するか
    async fn has_steep_slope_nearby(
        &self,
        coord: &Coord,
    ) -> Result<bool, DomainError>;

    /// 半径 1km 内の学校数 + 最寄り距離(m)
    async fn count_schools_nearby(
        &self,
        coord: &Coord,
    ) -> Result<(i64, f64), DomainError>;

    /// 半径 1km 内の医療機関数 + 最寄り距離(m)
    async fn count_medical_nearby(
        &self,
        coord: &Coord,
    ) -> Result<(i64, f64), DomainError>;
}

// ─── Stats ───────────────────────────────────────────

#[async_trait]
pub trait StatsRepository: Send + Sync {
    async fn calc_land_price_stats(
        &self,
        bbox: &BBox,
    ) -> Result<LandPriceStats, DomainError>;

    async fn calc_risk_stats(
        &self,
        bbox: &BBox,
    ) -> Result<RiskStats, DomainError>;

    async fn count_facilities(
        &self,
        bbox: &BBox,
    ) -> Result<FacilityStats, DomainError>;

    async fn calc_zoning_distribution(
        &self,
        bbox: &BBox,
    ) -> Result<HashMap<String, f64>, DomainError>;
}

// ─── Trend ───────────────────────────────────────────

#[async_trait]
pub trait TrendRepository: Send + Sync {
    /// 最寄り観測地点の地価推移データ（2km 以内）
    async fn find_trend(
        &self,
        coord: &Coord,
        years: i32,
    ) -> Result<Option<(TrendLocation, Vec<TrendPoint>)>, DomainError>;
}

// ─── Health ──────────────────────────────────────────

#[async_trait]
pub trait HealthRepository: Send + Sync {
    /// DB 接続確認（SELECT 1）
    async fn check_connection(&self) -> bool;
}
```

`★ Insight ─────────────────────────────────────`
**Repository trait 設計の原則:**

1. **Domain 層に定義、Infra 層で実装** — 依存性逆転の原則。Domain は「何が必要か」を宣言し、Infra が「どう取るか」を実装
2. **`&self` メソッド** — フリー関数（`fn query_x(pool, ...)`) ではなく `&self` メソッドにすることで `Arc<dyn Trait>` として DI 可能
3. **`Send + Sync` 境界** — Axum のハンドラは `Send + 'static` が必要。tokio の `spawn` にも必要
4. **`async_trait`** — Rust の async fn in trait は static dispatch のみ対応（1.75+）。`dyn Trait` で使うには `async_trait` クレートが必要
5. **AreaRepository のレイヤー分割** — 各テーブルのスキーマが異なるため、レイヤーごとにメソッドを分割。将来の固有型への拡張性を確保
`─────────────────────────────────────────────────`

### 3.4 ドメインエラー — `domain/error.rs`

```rust
/// フレームワーク非依存のドメインエラー。
/// HTTP ステータスコードへの変換は handler 層の責務。
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Invalid coordinate: {0}")]
    InvalidCoordinate(String),

    #[error("Bounding box exceeds maximum allowed area (0.5 degrees per side)")]
    BBoxTooLarge,

    #[error("Required parameter missing: {0}")]
    MissingParameter(String),

    #[error("Resource not found")]
    NotFound,

    #[error("Database error: {0}")]
    Database(String),

    #[error("Unexpected error: {0}")]
    Unexpected(String),
}
```

**ポイント:**
- `sqlx::Error` や `anyhow::Error` に直接依存しない
- `From<sqlx::Error>` は **Infra 層で実装** し、`DomainError::Database(e.to_string())` に変換
- `IntoResponse` は **Handler 層で実装**（Domain は HTTP を知らない）

---

## 4. Usecase 層（Domain のみに依存）

### 4.1 `usecase/get_area_data.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::GeoFeature;
use crate::domain::error::DomainError;
use crate::domain::repository::AreaRepository;
use crate::domain::value_object::{BBox, LayerType};

pub struct GetAreaDataUsecase {
    repo: Arc<dyn AreaRepository>,
}

impl GetAreaDataUsecase {
    pub fn new(repo: Arc<dyn AreaRepository>) -> Self {
        Self { repo }
    }

    /// BBox 内の指定レイヤーデータを取得。
    /// 不明レイヤーはスキップし、ログに警告を出力。
    #[tracing::instrument(skip(self))]
    pub async fn execute(
        &self,
        bbox: &BBox,
        layer_names: &[String],
    ) -> Result<HashMap<LayerType, Vec<GeoFeature>>, DomainError> {
        if layer_names.is_empty() {
            return Err(DomainError::MissingParameter("layers".into()));
        }

        let layers: Vec<LayerType> = layer_names
            .iter()
            .filter_map(|s| {
                let layer = LayerType::from_str(s);
                if layer.is_none() {
                    tracing::warn!(layer = %s, "unknown layer requested, skipping");
                }
                layer
            })
            .collect();

        let mut result = HashMap::new();
        for layer in layers {
            let features = self.dispatch(bbox, layer).await?;
            result.insert(layer, features);
        }

        Ok(result)
    }

    /// LayerType に応じて AreaRepository の対応メソッドを呼び出す。
    /// Usecase 層の責務: レイヤー名 → リポジトリメソッドのルーティング。
    async fn dispatch(
        &self,
        bbox: &BBox,
        layer: LayerType,
    ) -> Result<Vec<GeoFeature>, DomainError> {
        match layer {
            LayerType::LandPrice  => self.repo.find_land_prices(bbox).await,
            LayerType::Zoning     => self.repo.find_zoning(bbox).await,
            LayerType::Flood      => self.repo.find_flood_risk(bbox).await,
            LayerType::SteepSlope => self.repo.find_steep_slope(bbox).await,
            LayerType::Schools    => self.repo.find_schools(bbox).await,
            LayerType::Medical    => self.repo.find_medical(bbox).await,
        }
    }
}
```

### 4.2 `usecase/compute_score.rs`

```rust
use std::sync::Arc;
use serde_json::json;

use crate::domain::entity::PriceRecord;
use crate::domain::error::DomainError;
use crate::domain::repository::ScoreRepository;
use crate::domain::value_object::*;

pub struct ComputeScoreUsecase {
    repo: Arc<dyn ScoreRepository>,
}

impl ComputeScoreUsecase {
    pub fn new(repo: Arc<dyn ScoreRepository>) -> Self {
        Self { repo }
    }

    /// 4コンポーネントを並列計算し、投資スコア (0-100) を返す。
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, coord: &Coord) -> Result<InvestmentScore, DomainError> {
        // NFR-1: 並列クエリ (target < 500ms)
        let (prices, flood, steep, schools, medical) = tokio::try_join!(
            self.repo.find_nearest_prices(coord),
            self.repo.calc_flood_overlap(coord),
            self.repo.has_steep_slope_nearby(coord),
            self.repo.count_schools_nearby(coord),
            self.repo.count_medical_nearby(coord),
        )?;

        Ok(InvestmentScore {
            trend: Self::calc_trend(&prices),
            risk: Self::calc_risk(flood, steep),
            access: Self::calc_access(schools, medical),
            yield_potential: Self::calc_yield(&prices),
        })
    }

    // --- 純粋関数: ビジネスロジック ---

    fn calc_trend(prices: &[PriceRecord]) -> ScoreComponent { /* ... */ }
    fn calc_risk(flood_overlap: f64, steep_nearby: bool) -> ScoreComponent { /* ... */ }
    fn calc_access(schools: (i64, f64), medical: (i64, f64)) -> ScoreComponent { /* ... */ }
    fn calc_yield(prices: &[PriceRecord]) -> ScoreComponent { /* ... */ }
}
```

**ポイント:**
- `calc_trend` 等は `&self` を取らない純粋関数 → 単体テスト容易
- ビジネスロジック（CAGR 計算、リスク合成、アクセススコア）は全て **Usecase 内**
- 現在 `services/scoring.rs` にあるロジックをそのまま移動

### 4.3 `usecase/get_stats.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::entity::*;
use crate::domain::error::DomainError;
use crate::domain::repository::StatsRepository;
use crate::domain::value_object::BBox;

pub struct AreaStats {
    pub land_price: LandPriceStats,
    pub risk: RiskStats,
    pub facilities: FacilityStats,
    pub zoning_distribution: HashMap<String, f64>,
}

pub struct GetStatsUsecase {
    repo: Arc<dyn StatsRepository>,
}

impl GetStatsUsecase {
    pub fn new(repo: Arc<dyn StatsRepository>) -> Self {
        Self { repo }
    }

    /// BBox 内のエリア統計を並列取得。
    #[tracing::instrument(skip(self))]
    pub async fn execute(&self, bbox: &BBox) -> Result<AreaStats, DomainError> {
        let (land_price, risk, facilities, zoning) = tokio::try_join!(
            self.repo.calc_land_price_stats(bbox),
            self.repo.calc_risk_stats(bbox),
            self.repo.count_facilities(bbox),
            self.repo.calc_zoning_distribution(bbox),
        )?;

        Ok(AreaStats { land_price, risk, facilities, zoning_distribution: zoning })
    }
}
```

### 4.4 `usecase/get_trend.rs`

```rust
use std::sync::Arc;

use crate::domain::error::DomainError;
use crate::domain::repository::TrendRepository;
use crate::domain::value_object::*;

pub struct GetTrendUsecase {
    repo: Arc<dyn TrendRepository>,
}

impl GetTrendUsecase {
    pub fn new(repo: Arc<dyn TrendRepository>) -> Self {
        Self { repo }
    }

    /// 最寄り観測地点の地価推移 + CAGR を算出。
    #[tracing::instrument(skip(self))]
    pub async fn execute(
        &self,
        coord: &Coord,
        years: i32,
    ) -> Result<TrendAnalysis, DomainError> {
        let years = years.clamp(1, 20);

        let (location, data) = self.repo
            .find_trend(coord, years)
            .await?
            .ok_or(DomainError::NotFound)?;

        if data.is_empty() {
            return Err(DomainError::NotFound);
        }

        let cagr = Self::calc_cagr(&data);
        let direction = if cagr > 0.0 { TrendDirection::Up } else { TrendDirection::Down };

        Ok(TrendAnalysis {
            location,
            data,
            cagr: (cagr * 1000.0).round() / 1000.0,
            direction,
        })
    }

    /// 純粋関数: CAGR = (last/first)^(1/years) - 1
    fn calc_cagr(data: &[TrendPoint]) -> f64 {
        if data.len() < 2 {
            return 0.0;
        }
        let first = data[0].price_per_sqm as f64;
        let last = data[data.len() - 1].price_per_sqm as f64;
        let n = (data[data.len() - 1].year - data[0].year).max(1) as f64;
        (last / first).powf(1.0 / n) - 1.0
    }
}
```

### 4.5 `usecase/check_health.rs`

```rust
use std::sync::Arc;
use crate::domain::repository::HealthRepository;

pub struct HealthStatus {
    pub status: &'static str,
    pub db_connected: bool,
    pub reinfolib_key_set: bool,
    pub version: &'static str,
}

pub struct CheckHealthUsecase {
    repo: Arc<dyn HealthRepository>,
    reinfolib_key_set: bool,
}

impl CheckHealthUsecase {
    pub fn new(repo: Arc<dyn HealthRepository>, reinfolib_key_set: bool) -> Self {
        Self { repo, reinfolib_key_set }
    }

    pub async fn execute(&self) -> HealthStatus {
        let db_connected = self.repo.check_connection().await;
        HealthStatus {
            status: if db_connected { "ok" } else { "degraded" },
            db_connected,
            reinfolib_key_set: self.reinfolib_key_set,
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}
```

`★ Insight ─────────────────────────────────────`
**Usecase 層の設計原則（YAGNI + DRY）:**

1. **struct + `execute()` パターン** — Go の `usecase.Run()` に相当。1 ユースケース = 1 構造体。テスト時にモックリポジトリを注入可能
2. **`Arc<dyn Trait>` を保持** — trait object によるランタイム多態。ジェネリクスでもよいが、Axum の `State` との相性で `Arc<dyn>` がシンプル
3. **純粋関数の分離** — `calc_trend()`, `calc_cagr()` 等は `&self` を取らない。入力→出力のみでテスト可能。I/O は `execute()` に集約
4. **ビジネスロジックの正しい所在** — CAGR 計算が `trend.rs`(handler) から `GetTrendUsecase` に移動。handler は「HTTP → Domain → HTTP」の変換のみ
`─────────────────────────────────────────────────`

---

## 5. Infra 層（Domain trait の実装）

### 5.1 設計方針

| 方針 | 内容 |
|------|------|
| **1 trait = 1 struct** | `PgAreaRepository`, `PgScoreRepository`, etc. |
| **sqlx::Error → DomainError** | `impl From<sqlx::Error> for DomainError` をこの層で定義 |
| **SQL は Infra 内に閉じる** | Domain/Usecase は SQL を一切知らない |
| **`PgPool` はコンストラクタ注入** | `PgXxxRepository::new(pool: PgPool)` |

### 5.2 `infra/pg_area_repository.rs`（例示）

```rust
use async_trait::async_trait;
use sqlx::PgPool;

use crate::domain::entity::GeoFeature;
use crate::domain::error::DomainError;
use crate::domain::repository::AreaRepository;
use crate::domain::value_object::BBox;

pub struct PgAreaRepository {
    pool: PgPool,
}

impl PgAreaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AreaRepository for PgAreaRepository {
    #[tracing::instrument(skip(self))]
    async fn find_land_prices(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        // 既存 repo_area::query_land_prices の SQL をここに移動
        let rows = sqlx::query!(
            r#"SELECT id, price_per_sqm, address, land_use, year,
                      ST_AsGeoJSON(geom)::jsonb AS geometry
               FROM land_prices
               WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))"#,
            bbox.west(), bbox.south(), bbox.east(), bbox.north(),
        )
        .fetch_all(&self.pool)
        .await?;

        // Row → GeoFeature 変換
        Ok(rows.into_iter().map(|r| /* ... */ ).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn find_zoning(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        // zoning テーブル固有のカラム (zone_type, zone_code, floor_area_ratio, ...)
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn find_flood_risk(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn find_steep_slope(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn find_schools(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        todo!()
    }

    #[tracing::instrument(skip(self))]
    async fn find_medical(&self, bbox: &BBox) -> Result<Vec<GeoFeature>, DomainError> {
        todo!()
    }
}
```

**分割の利点:** 各メソッドが固有のテーブルスキーマに最適化された SQL を持つ。将来レイヤー固有の戻り値型（例: `LandPriceFeature`）に変更する際も、1メソッドずつ段階的に移行可能。

### 5.3 sqlx::Error → DomainError 変換

```rust
// infra/mod.rs に配置
impl From<sqlx::Error> for DomainError {
    fn from(e: sqlx::Error) -> Self {
        tracing::error!(error = %e, "database query failed");
        DomainError::Database(e.to_string())
    }
}
```

---

## 6. Handler 層（Axum 固有コード）

### 6.1 リクエスト DTO — `handler/request.rs`

```rust
use serde::Deserialize;
use crate::domain::value_object::{BBox, Coord};
use crate::domain::error::DomainError;

/// BBox クエリパラメータ（Axum Query 用）
#[derive(Debug, Deserialize)]
pub struct BBoxQuery {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
}

impl BBoxQuery {
    /// DTO → Domain Value Object への変換（バリデーション含む）
    pub fn into_domain(self) -> Result<BBox, DomainError> {
        BBox::new(self.south, self.west, self.north, self.east)
    }
}

/// 座標クエリパラメータ
#[derive(Debug, Deserialize)]
pub struct CoordQuery {
    pub lat: f64,
    pub lng: f64,
}

impl CoordQuery {
    pub fn into_domain(self) -> Result<Coord, DomainError> {
        Coord::new(self.lat, self.lng)
    }
}

/// area-data 用（bbox + layers）
#[derive(Debug, Deserialize)]
pub struct AreaDataQuery {
    pub south: f64,
    pub west: f64,
    pub north: f64,
    pub east: f64,
    pub layers: String,
}

/// trend 用（coord + optional years）
#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    pub lat: f64,
    pub lng: f64,
    #[serde(default = "default_years")]
    pub years: i32,
}

fn default_years() -> i32 { 5 }
```

`★ Insight ─────────────────────────────────────`
**DTO → Domain 変換パターン（Boundary Validation）:**

- `BBoxQuery`（Handler DTO）と `BBox`（Domain Value Object）を**分離**
- `into_domain()` メソッドで変換時にバリデーション実行
- **DTO は `Deserialize` のみ**（Axum 用）、**Domain 型は `Deserialize` を持たない**（HTTP 非依存）
- これにより「area-data と stats は同じ `BBox` を使うが、Handler の受け取り方が異なる」ケースに対応
- 現在の問題：`BBoxParams` が `Deserialize` + `validate()` を両方持ち、Axum 層と Domain 層の責務が混在
`─────────────────────────────────────────────────`

### 6.2 レスポンス DTO — `handler/response.rs`

```rust
use serde::Serialize;
use std::collections::HashMap;

use crate::domain::entity::GeoFeature;
use crate::domain::value_object::*;
use crate::usecase::get_stats::AreaStats;
use crate::usecase::check_health::HealthStatus;

// ─── GeoJSON ─────────────────────────────

#[derive(Serialize)]
pub struct FeatureCollectionJson {
    pub r#type: &'static str,
    pub features: Vec<FeatureJson>,
}

#[derive(Serialize)]
pub struct FeatureJson {
    pub r#type: &'static str,
    pub geometry: serde_json::Value,
    pub properties: serde_json::Value,
}

impl From<GeoFeature> for FeatureJson {
    fn from(f: GeoFeature) -> Self {
        Self {
            r#type: "Feature",
            geometry: serde_json::json!({
                "type": f.geometry.r#type,
                "coordinates": f.geometry.coordinates,
            }),
            properties: f.properties,
        }
    }
}

impl FeatureCollectionJson {
    pub fn from_features(features: Vec<GeoFeature>) -> Self {
        Self {
            r#type: "FeatureCollection",
            features: features.into_iter().map(FeatureJson::from).collect(),
        }
    }
}

// ─── Score ───────────────────────────────

#[derive(Serialize)]
pub struct ScoreJson {
    pub score: f64,
    pub components: ScoreComponentsJson,
    pub metadata: ScoreMetadataJson,
}

#[derive(Serialize)]
pub struct ScoreComponentsJson {
    pub trend: ScoreDetailJson,
    pub risk: ScoreDetailJson,
    pub access: ScoreDetailJson,
    pub yield_potential: ScoreDetailJson,
}

#[derive(Serialize)]
pub struct ScoreDetailJson {
    pub value: f64,
    pub max: f64,
    pub detail: serde_json::Value,
}

#[derive(Serialize)]
pub struct ScoreMetadataJson {
    pub calculated_at: String,
    pub data_freshness: String,
    pub disclaimer: String,
}

impl From<InvestmentScore> for ScoreJson {
    fn from(s: InvestmentScore) -> Self {
        Self {
            score: s.total(),
            components: ScoreComponentsJson {
                trend: s.trend.into(),
                risk: s.risk.into(),
                access: s.access.into(),
                yield_potential: s.yield_potential.into(),
            },
            metadata: ScoreMetadataJson {
                calculated_at: chrono::Utc::now().to_rfc3339(),
                data_freshness: "2024".into(), // TODO: 動的に取得
                disclaimer: "本スコアは参考値です。投資判断は自己責任で行ってください。".into(),
            },
        }
    }
}

impl From<ScoreComponent> for ScoreDetailJson {
    fn from(c: ScoreComponent) -> Self {
        Self { value: c.value, max: c.max, detail: c.detail }
    }
}

// ─── Stats ───────────────────────────────

#[derive(Serialize)]
pub struct StatsJson {
    pub land_price: LandPriceStatsJson,
    pub risk: RiskStatsJson,
    pub facilities: FacilityStatsJson,
    pub zoning_distribution: HashMap<String, f64>,
}

// ...各サブ型は Domain entity から From 変換で生成

// ─── Trend ───────────────────────────────

#[derive(Serialize)]
pub struct TrendJson {
    pub location: TrendLocationJson,
    pub data: Vec<TrendPointJson>,
    pub cagr: f64,
    pub direction: String,
}

impl From<TrendAnalysis> for TrendJson {
    fn from(t: TrendAnalysis) -> Self {
        Self {
            location: TrendLocationJson {
                address: t.location.address,
                distance_m: t.location.distance_m,
            },
            data: t.data.into_iter().map(|p| TrendPointJson {
                year: p.year,
                price_per_sqm: p.price_per_sqm,
            }).collect(),
            cagr: t.cagr,
            direction: t.direction.as_str().into(),
        }
    }
}

// ─── Health ──────────────────────────────

#[derive(Serialize)]
pub struct HealthJson {
    pub status: String,
    pub db_connected: bool,
    pub reinfolib_key_set: bool,
    pub version: String,
}

impl From<HealthStatus> for HealthJson {
    fn from(h: HealthStatus) -> Self {
        Self {
            status: h.status.into(),
            db_connected: h.db_connected,
            reinfolib_key_set: h.reinfolib_key_set,
            version: h.version.into(),
        }
    }
}
```

### 6.3 エラー変換 — `handler/error.rs`

```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

use crate::domain::error::DomainError;

/// Handler 層のエラー型。DomainError をラップし HTTP レスポンスに変換。
pub struct ApiError(pub DomainError);

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self { Self(e) }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = match &self.0 {
            DomainError::InvalidCoordinate(_) => (StatusCode::BAD_REQUEST, "INVALID_PARAMS"),
            DomainError::BBoxTooLarge          => (StatusCode::BAD_REQUEST, "BBOX_TOO_LARGE"),
            DomainError::MissingParameter(_)   => (StatusCode::BAD_REQUEST, "INVALID_PARAMS"),
            DomainError::NotFound              => (StatusCode::NOT_FOUND, "NOT_FOUND"),
            DomainError::Database(_)           => (StatusCode::SERVICE_UNAVAILABLE, "DB_UNAVAILABLE"),
            DomainError::Unexpected(_)         => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR"),
        };

        let body = json!({
            "error": {
                "code": code,
                "message": self.0.to_string(),
            }
        });

        (status, Json(body)).into_response()
    }
}
```

### 6.4 Handler 例: `handler/score.rs`

```rust
use axum::{extract::{Query, State}, Json};
use crate::handler::error::ApiError;
use crate::handler::request::CoordQuery;
use crate::handler::response::ScoreJson;
use crate::handler::AppState;

/// `GET /api/score?lat=35.68&lng=139.76`
#[tracing::instrument(skip(state))]
pub async fn get_score(
    State(state): State<AppState>,
    Query(query): Query<CoordQuery>,
) -> Result<Json<ScoreJson>, ApiError> {
    let coord = query.into_domain()?;                          // DTO → Domain
    let score = state.score_usecase.execute(&coord).await?;    // Usecase 呼び出し
    Ok(Json(ScoreJson::from(score)))                           // Domain → DTO
}
```

**Handler の責務は3行:**
1. **DTO → Domain** 変換（バリデーション含む）
2. **Usecase** 呼び出し
3. **Domain → DTO** 変換（シリアライズ含む）

### 6.5 Handler 例: `handler/area_data.rs`

```rust
use axum::{extract::{Query, State}, Json};
use serde_json::Value;

use crate::handler::error::ApiError;
use crate::handler::request::AreaDataQuery;
use crate::handler::response::FeatureCollectionJson;
use crate::handler::AppState;

/// `GET /api/area-data?south=&west=&north=&east=&layers=landprice,zoning,...`
#[tracing::instrument(skip(state))]
pub async fn get_area_data(
    State(state): State<AppState>,
    Query(query): Query<AreaDataQuery>,
) -> Result<Json<Value>, ApiError> {
    let bbox = crate::domain::value_object::BBox::new(
        query.south, query.west, query.north, query.east,
    )?;

    let layer_names: Vec<String> = query.layers
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let features_map = state.area_data_usecase.execute(&bbox, &layer_names).await?;

    // Domain → JSON レスポンス
    let mut result = serde_json::Map::new();
    for (layer_type, features) in features_map {
        let fc = FeatureCollectionJson::from_features(features);
        result.insert(
            layer_type.as_str().to_string(),
            serde_json::to_value(fc).map_err(|e| {
                ApiError(crate::domain::error::DomainError::Unexpected(e.to_string()))
            })?,
        );
    }

    Ok(Json(Value::Object(result)))
}
```

---

## 7. DI Wiring — `main.rs`

### 7.1 新 AppState

```rust
use std::sync::Arc;

use crate::usecase::{
    check_health::CheckHealthUsecase,
    compute_score::ComputeScoreUsecase,
    get_area_data::GetAreaDataUsecase,
    get_stats::GetStatsUsecase,
    get_trend::GetTrendUsecase,
};

#[derive(Clone)]
pub struct AppState {
    pub health_usecase: Arc<CheckHealthUsecase>,
    pub area_data_usecase: Arc<GetAreaDataUsecase>,
    pub score_usecase: Arc<ComputeScoreUsecase>,
    pub stats_usecase: Arc<GetStatsUsecase>,
    pub trend_usecase: Arc<GetTrendUsecase>,
}
```

### 7.2 DI 組み立て

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::from_env();
    logging::init(&config);

    let pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await?;

    // --- Repository（Infra 層の具象型を生成）---
    let area_repo = Arc::new(PgAreaRepository::new(pool.clone()));
    let score_repo = Arc::new(PgScoreRepository::new(pool.clone()));
    let stats_repo = Arc::new(PgStatsRepository::new(pool.clone()));
    let trend_repo = Arc::new(PgTrendRepository::new(pool.clone()));
    let health_repo = Arc::new(PgHealthRepository::new(pool.clone()));

    // --- Usecase（Repository を注入）---
    let state = AppState {
        health_usecase: Arc::new(CheckHealthUsecase::new(
            health_repo, config.reinfolib_api_key.is_some(),
        )),
        area_data_usecase: Arc::new(GetAreaDataUsecase::new(area_repo)),
        score_usecase: Arc::new(ComputeScoreUsecase::new(score_repo)),
        stats_usecase: Arc::new(GetStatsUsecase::new(stats_repo)),
        trend_usecase: Arc::new(GetTrendUsecase::new(trend_repo)),
    };

    // --- Router ---
    let app = Router::new()
        .route("/api/health", get(handler::health::health))
        .route("/api/area-data", get(handler::area_data::get_area_data))
        .route("/api/score", get(handler::score::get_score))
        .route("/api/stats", get(handler::stats::get_stats))
        .route("/api/trend", get(handler::trend::get_trend))
        .with_state(state)
        .layer(logging::http_trace_layer())
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

`★ Insight ─────────────────────────────────────`
**DI の「組み立て」は `main.rs` の責務:**

これは **Composition Root** パターン。アプリケーション全体の依存グラフを1箇所で組み立てる：
- `PgPool` → `PgXxxRepository::new(pool)` → `XxxUsecase::new(repo)` → `AppState`
- テスト時は `MockXxxRepository::new()` → `XxxUsecase::new(mock)` で差し替え可能
- Handler は `AppState` 経由で Usecase を使うだけ。Infra の具象型を知らない

**`Arc` を使う理由:**
- Axum の `State<T>` は `Clone` を要求する
- `dyn Trait` は `Clone` できないため `Arc` でラップ
- `Arc::clone()` はポインタのコピーのみ（O(1)）、パフォーマンス影響なし
`─────────────────────────────────────────────────`

---

## 8. 依存関係まとめ

```
┌─────────────────────────────────────────────────────┐
│  handler/         Axum 依存 (axum, serde, tower-http)│
│   ├── request.rs     Deserialize → Domain 値変換    │
│   ├── response.rs    Domain → Serialize 変換        │
│   ├── error.rs       DomainError → HTTP Response    │
│   └── *.rs           3行: parse → usecase → format  │
├─────────────────────────────────────────────────────┤
│  usecase/         Domain のみに依存                   │
│   └── *.rs           Repository trait + Domain 型    │
├─────────────────────────────────────────────────────┤
│  domain/          外部依存ゼロ (thiserror のみ)       │
│   ├── entity.rs      ビジネスエンティティ             │
│   ├── value_object.rs  不変条件付き値オブジェクト     │
│   ├── repository.rs  async_trait Repository 定義     │
│   └── error.rs       フレームワーク非依存エラー       │
├─────────────────────────────────────────────────────┤
│  infra/           Domain trait を実装                 │
│   └── pg_*.rs        sqlx + PostGIS クエリ           │
│                      sqlx::Error → DomainError 変換  │
└─────────────────────────────────────────────────────┘
```

**依存方向:** `handler → usecase → domain ← infra`
Domain は**何にも依存しない**。Infra は Domain の trait を実装する（依存性逆転）。

---

## 9. Cargo.toml 変更

```toml
[dependencies]
# 追加
async-trait = "0.1"     # dyn dispatch 対応の async trait

# 既存（変更なし）
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "json", "migrate"] }
tower-http = { version = "0.6", features = ["cors", "compression-gzip", "trace"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json", "fmt"] }
dotenvy = "0.15"
envy = "0.4"
thiserror = "2"
anyhow = "1"
chrono = { version = "0.4", features = ["serde"] }
```

---

## 10. エンドポイント設計（変更なし）

REST API のパス・パラメータ・レスポンス形式は変更しない（FE 互換維持）。

| Method | Path | Handler | Usecase | Repository |
|--------|------|---------|---------|------------|
| GET | `/api/health` | `handler::health` | `CheckHealthUsecase` | `HealthRepository` |
| GET | `/api/area-data` | `handler::area_data` | `GetAreaDataUsecase` | `AreaRepository` |
| GET | `/api/score` | `handler::score` | `ComputeScoreUsecase` | `ScoreRepository` |
| GET | `/api/stats` | `handler::stats` | `GetStatsUsecase` | `StatsRepository` |
| GET | `/api/trend` | `handler::trend` | `GetTrendUsecase` | `TrendRepository` |

---

## 11. 実装タスク

- [ ] Task 1: `domain/` 層を作成（entity, value_object, repository trait, error）→ Verify: `cargo check`
- [ ] Task 2: `usecase/` 層を作成（5 usecase 構造体）→ Verify: `cargo check`
- [ ] Task 3: `infra/` 層をリファクタ（フリー関数 → trait impl）→ Verify: `cargo check`
- [ ] Task 4: `handler/` 層を作成（request/response DTO, error 変換）→ Verify: `cargo check`
- [ ] Task 5: `main.rs` の DI wiring → Verify: `cargo clippy -- -D warnings`
- [ ] Task 6: テスト更新 + Usecase 単体テスト追加 → Verify: `cargo test`
- [ ] Task 7: 最終検証（`cargo build && cargo clippy && cargo test`）

## Done When

- [ ] `cargo clippy -- -D warnings` がクリーン
- [ ] `cargo test` が全パス
- [ ] Domain 層に `use axum` / `use sqlx` が存在しない
- [ ] Handler の各関数が 5行以内（parse → usecase → format）
- [ ] 全ての Repository trait にモック実装可能（テストで確認）

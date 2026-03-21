# 共通ライブラリクレート抽出計画

## Goal

`services/backend/src/` 内の重複コードと再利用可能な機能を `lib/` 配下の独立クレートに抽出し、コードの DRY 化・テスタビリティ向上・将来の複数バイナリ対応を実現する。

## 現状分析（抽出候補の根拠）

| パターン | 出現回数 | リスク | 場所 |
|---------|---------|--------|------|
| `map_db_err(sqlx::Error)` 関数コピー | 4箇所 | バグ混入 | pg_{area,score,stats,trend}_repository.rs |
| `ST_MakeEnvelope($1,$2,$3,$4,4326)` SQL + `.bind(bbox.west/south/east/north)` | 11箇所 | パラメータ順序ミス | pg_area_repository(6), pg_stats_repository(5) |
| `ST_MakePoint($1,$2)` + `.bind(coord.lng/lat)` | 7箇所 | lng/lat逆転 | pg_score_repository(5), pg_trend_repository(2) |
| `to_geo_feature()` PostGIS→Domain変換 | 1箇所 | 将来コピー増殖 | pg_area_repository.rs |
| CAGR計算ロジック | 2箇所 | 結果不一致 | compute_score.rs, get_trend.rs |
| `round_dp` 丸め処理 `(x * 10^n).round() / 10^n` | 6箇所 | 精度不統一 | compute_score.rs |
| `FeatureCollectionDto` GeoJSON組立 | 1箇所 | 将来コピー | handler/response.rs |
| `AppError` DomainError→HTTP変換 | 1箇所 | 2nd binary時コピー | handler/error.rs |
| reinfolib/e-Stat APIクライアント | 未実装 | Phase 2必須 | config.rsにキーのみ |

## 抽出するクレート一覧

```
services/backend/
├── Cargo.toml                          ← workspace root
├── lib/
│   ├── telemetry/                      ← 既存 ✅
│   ├── db/                             ← NEW: PostGIS共通クライアント
│   ├── axum-support/                   ← NEW: Axumサーバー共通処理
│   ├── geo-math/                       ← NEW: 空間・金融計算ユーティリティ
│   └── mlit-client/                    ← NEW: 行政API統合クライアント
└── src/                                ← realestate-api binary
```

---

## Crate 1: `realestate-db` — PostGIS共通クライアント

**Cargo name**: `realestate-db`
**Path**: `lib/db/`
**責任**: SQLx/PostGIS操作の共通ユーティリティ。全リポジトリ実装が依存する基盤。

### 公開API

```rust
// lib/db/src/lib.rs
pub mod error;    // sqlx::Error → DomainError 変換
pub mod spatial;  // PostGIS バインドヘルパー
pub mod geo;      // GeoJSON変換ユーティリティ
pub mod pool;     // PgPool設定ヘルパー
```

### モジュール詳細

**`error.rs`** — DB エラー変換（4箇所の重複解消）
```rust
/// sqlx::Error を DomainError::Database に変換する。
/// 全 Pg*Repository で `.map_err(map_db_err)` として使用。
pub fn map_db_err(e: sqlx::Error) -> DomainError;

/// sqlx::Error をトレースログ付きで変換する。
/// クエリ名をログに含めることでデバッグ性向上。
pub fn map_db_err_with_context(e: sqlx::Error, query_name: &str) -> DomainError;
```

**`spatial.rs`** — PostGIS バインドヘルパー（11+7箇所の重複解消）
```rust
/// BBox の4パラメータを正しい順序（west, south, east, north = xmin, ymin, xmax, ymax）で
/// クエリにバインドするヘルパー。パラメータ順序ミスを型レベルで防止。
pub fn bind_bbox<'q, O>(
    query: sqlx::query::QueryAs<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>,
    bbox: &BBox,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>;

/// Coord のlng/latを正しい順序でバインド。
/// PostGIS ST_MakePoint は (longitude, latitude) = (x, y) の順。
pub fn bind_coord<'q, O>(
    query: sqlx::query::QueryAs<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>,
    coord: &Coord,
) -> sqlx::query::QueryAs<'q, sqlx::Postgres, O, sqlx::postgres::PgArguments>;

/// ST_MakeEnvelope SQL フラグメントを生成。
/// `placeholder_start` でプレースホルダー番号を指定可能。
pub fn st_envelope_sql(placeholder_start: u8) -> String;

/// ST_DWithin SQL フラグメントを生成。
pub fn st_dwithin_sql(placeholder_start: u8, radius_m: u32) -> String;
```

**`geo.rs`** — GeoJSON 変換（`to_geo_feature` 統合）
```rust
/// PostGIS ST_AsGeoJSON の結果を domain GeoFeature に変換する。
/// 全リポジトリで共通利用。
pub fn to_geo_feature(geojson: serde_json::Value, properties: serde_json::Value) -> GeoFeature;

/// serde_json::Value から GeoJSON geometry type を安全に抽出。
pub fn extract_geometry_type(geojson: &serde_json::Value) -> &str;
```

**`pool.rs`** — PgPool 設定
```rust
/// PgPool を構築する。接続URL、最大接続数、タイムアウトを設定。
/// main.rs の PgPoolOptions 設定を一元管理。
pub async fn create_pool(database_url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error>;
```

### 依存関係

```toml
[dependencies]
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "json"] }
serde_json = "1"
tracing = "0.1"
# domain types は realestate-api 側の domain/ に残す → trait で抽象化
```

### 設計判断

- **DomainError への依存**: `map_db_err` は `DomainError` を返す。選択肢は2つ:
  - (A) `realestate-db` が domain クレートに依存 → 循環リスク
  - (B) `realestate-db` は汎用エラー型を返し、呼び出し側で `.map_err()` → 安全だが冗長
  - **採用: (B)** — `realestate-db` は `DbError(sqlx::Error)` を返し、infra層で `.map_err(DbError::into_domain)` を呼ぶ。domain層の純粋性を維持。

→ Verify: `cargo test -p realestate-db` で spatial バインドヘルパーのユニットテスト通過

---

## Crate 2: `realestate-axum-support` — Axumサーバー共通処理

**Cargo name**: `realestate-axum-support`
**Path**: `lib/axum-support/`
**責任**: Axum HTTPサーバーの共通ミドルウェアとエラーハンドリング。handler 層のボイラープレート削減。

### 公開API

```rust
// lib/axum-support/src/lib.rs
pub mod error;     // AppError + DomainError→HTTP 変換
pub mod response;  // GeoJSON FeatureCollection DTO
pub mod extract;   // 共通 Extractor ユーティリティ
```

### モジュール詳細

**`error.rs`** — 統一エラーレスポンス
```rust
/// DomainError → HTTP レスポンス変換。
/// JSON body: `{ "error": { "code": "MACHINE_CODE", "message": "..." } }`
pub struct AppError(DomainError);

impl IntoResponse for AppError { ... }
impl From<DomainError> for AppError { ... }

/// エラーコードと HTTP ステータスのマッピングテーブル。
/// DomainError の各バリアントに対応。
pub trait ErrorMapping {
    fn status_code(&self) -> StatusCode;
    fn error_code(&self) -> &'static str;
}
```

**`response.rs`** — GeoJSON レスポンス型
```rust
/// RFC 7946 準拠の FeatureCollection シリアライズ。
/// handler/response.rs の FeatureCollectionDto/FeatureDto/GeometryDto を移行。
pub struct FeatureCollectionDto { ... }
pub struct FeatureDto { ... }
pub struct GeometryDto { ... }

impl FeatureCollectionDto {
    pub fn from_features(features: Vec<GeoFeature>) -> Self;
}
```

**`extract.rs`** — 共通 Extractor
```rust
/// validated BBox を Axum Query から抽出する型エイリアス。
/// 将来: カスタム Extractor で into_domain() を自動呼び出し。
// Phase 1 ではユーティリティ関数のみ
```

### 依存関係

```toml
[dependencies]
axum = "0.8"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### 設計判断

- `AppError` が `DomainError` に依存する問題: `DomainError` は domain 層の型。
  - **採用**: `realestate-axum-support` はジェネリックな `ApiError<E: ErrorMapping>` を提供。API固有の `AppError(DomainError)` は `realestate-api` 側で `ErrorMapping` を実装する。

→ Verify: `cargo test -p realestate-axum-support` でエラーレスポンスJSONフォーマットのテスト通過

---

## Crate 3: `realestate-geo-math` — 空間・金融計算ユーティリティ

**Cargo name**: `realestate-geo-math`
**Path**: `lib/geo-math/`
**責任**: 純粋関数の計算ライブラリ。I/O 依存ゼロ。domain 層の計算ロジックを集約。

### 公開API

```rust
// lib/geo-math/src/lib.rs
pub mod finance;   // CAGR、利回り計算
pub mod rounding;  // 精度制御ユーティリティ
pub mod spatial;   // bbox面積計算など
```

### モジュール詳細

**`finance.rs`** — 金融計算（2箇所のCAGR重複解消）
```rust
/// 年複利成長率 (CAGR) を計算する。
///
/// CAGR = (latest / oldest)^(1/years) - 1
///
/// # Arguments
/// * `oldest_price` — 期間開始時の価格（> 0）
/// * `latest_price` — 期間終了時の価格（> 0）
/// * `years` — 期間年数（> 0、0の場合は1に丸め）
///
/// # Returns
/// CAGR（例: 0.05 = 5%成長/年）
pub fn compute_cagr(oldest_price: f64, latest_price: f64, years: u32) -> f64;

/// 推定利回り（Phase 1: 取引価格 ≈ 地価の80%と仮定）
pub fn estimate_yield(land_price: i64, transaction_ratio: f64) -> f64;
```

**`rounding.rs`** — 精度制御（6箇所の丸め重複解消）
```rust
/// 指定した小数桁で丸める。
///
/// `round_dp(3.14159, 2)` → `3.14`
/// `round_dp(25.55, 1)` → `25.6`
///
/// API レスポンスの数値精度を一元管理。
pub fn round_dp(value: f64, decimal_places: u32) -> f64;
```

**`spatial.rs`** — 空間計算
```rust
/// BBox の面積を平方度で計算する（簡易）。
/// メトリクス `spatial.bbox.area_deg2` に使用。
pub fn bbox_area_deg2(south: f64, west: f64, north: f64, east: f64) -> f64;
```

### 依存関係

```toml
[dependencies]
# なし — 純粋計算のみ、外部依存ゼロ
```

### テスト方針

- プロパティベーステスト: `compute_cagr(p, p, n) == 0.0`（同一価格ならCAGR=0）
- エッジケース: `years=0`, `price=0`, 負の価格
- `round_dp` の IEEE 754 境界値テスト

→ Verify: `cargo test -p realestate-geo-math` — 全テスト通過 + 既存 compute_score テスト維持

---

## Crate 4: `mlit-client` — 行政API統合クライアント

**Cargo name**: `mlit-client`
**Path**: `lib/mlit-client/`
**責任**: 国交省 reinfolib API / 国土数値情報 / e-Stat API への HTTP クライアント。Phase 2 の reinfolib API 切替に備える。

### 公開API

```rust
// lib/mlit-client/src/lib.rs
pub mod reinfolib;  // 不動産情報ライブラリ API クライアント
pub mod ksj;        // 国土数値情報ダウンロードサービス
pub mod estat;      // e-Stat 政府統計 API クライアント
pub mod error;      // 統一エラー型
pub mod config;     // APIキー・エンドポイント設定
```

### モジュール詳細

**`reinfolib.rs`** — 不動産情報ライブラリ API（Phase 2 メイン）
```rust
/// reinfolib API クライアント。
/// https://www.reinfolib.mlit.go.jp/ の各エンドポイントを型安全にラップ。
pub struct ReinfolibClient {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

impl ReinfolibClient {
    pub fn new(api_key: String) -> Self;

    /// XPT001: 不動産取引価格ポイント
    pub async fn get_transaction_prices(&self, bbox: &BBox, year: i32) -> Result<Vec<TransactionPoint>, MlitError>;

    /// XPT002: 地価公示ポイント
    pub async fn get_land_prices(&self, bbox: &BBox, year: i32) -> Result<Vec<LandPricePoint>, MlitError>;

    /// XKT002: 用途地域ポリゴン
    pub async fn get_zoning(&self, bbox: &BBox) -> Result<Vec<ZoningPolygon>, MlitError>;

    /// XKT006: 学校ポイント
    pub async fn get_schools(&self, bbox: &BBox) -> Result<Vec<SchoolPoint>, MlitError>;

    /// XKT010: 医療機関ポイント
    pub async fn get_medical(&self, bbox: &BBox) -> Result<Vec<MedicalPoint>, MlitError>;
}
```

**`ksj.rs`** — 国土数値情報（Phase 1 データダウンロード用）
```rust
/// 国土数値情報 API クライアント。
/// GeoJSON ZIPファイルのダウンロードURL取得。
pub struct KsjClient {
    http: reqwest::Client,
}

impl KsjClient {
    /// 指定コード・都道府県・年度の GeoJSON URL を取得。
    pub async fn get_download_url(&self, code: &str, pref_code: &str, year: &str) -> Result<String, MlitError>;
}
```

**`estat.rs`** — e-Stat（将来の統計データ連携）
```rust
pub struct EstatClient {
    http: reqwest::Client,
    app_id: String,
}

impl EstatClient {
    /// 統計データを取得（Phase 3向け）
    pub async fn get_stats_data(&self, stats_data_id: &str) -> Result<serde_json::Value, MlitError>;
}
```

**`error.rs`** — 統一エラー
```rust
#[derive(Debug, thiserror::Error)]
pub enum MlitError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("API returned error: {status} {message}")]
    Api { status: u16, message: String },
    #[error("Response parse error: {0}")]
    Parse(String),
    #[error("Rate limited, retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },
}
```

**`config.rs`** — 設定
```rust
/// 行政API設定。環境変数から読み込み。
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MlitConfig {
    /// reinfolib API キー（REINFOLIB_API_KEY）
    pub reinfolib_api_key: Option<String>,
    /// e-Stat アプリID（ESTAT_APP_ID）
    pub estat_app_id: Option<String>,
    /// リクエストタイムアウト秒（デフォルト: 30）
    pub request_timeout_secs: Option<u64>,
}
```

### 依存関係

```toml
[features]
default = ["reinfolib"]
reinfolib = []
ksj = []
estat = []

[dependencies]
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tracing = "0.1"
tokio = { version = "1", features = ["time"] }  # リトライ用
```

### テスト方針

- モックHTTPサーバー（`wiremock`）でレスポンスを再現
- レートリミット時のリトライ動作テスト
- GeoJSON レスポンスのデシリアライゼーションテスト

→ Verify: `cargo test -p mlit-client` — モックテスト通過

---

## ワークスペース最終構成

```toml
# services/backend/Cargo.toml
[workspace]
members = [
    ".",
    "lib/telemetry",
    "lib/db",
    "lib/axum-support",
    "lib/geo-math",
    "lib/mlit-client",
]
```

### 依存関係グラフ

```
realestate-api (binary)
├── realestate-telemetry   ← ログ、メトリクス、HTTPトレース
├── realestate-db          ← PostGIS共通クライアント
├── realestate-axum-support ← HTTP共通処理
├── realestate-geo-math    ← 計算ユーティリティ
├── mlit-client            ← 行政APIクライアント
└── src/
    ├── domain/            ← 純粋ドメイン（外部依存ゼロ）
    ├── usecase/           ← realestate-geo-math に依存
    ├── infra/             ← realestate-db に依存
    └── handler/           ← realestate-axum-support に依存
```

**依存方向の鉄則**:
- `realestate-geo-math` → 依存ゼロ（純粋関数）
- `realestate-db` → sqlx のみ（domain型はジェネリック）
- `realestate-axum-support` → axum のみ（domain型はtrait抽象）
- `mlit-client` → reqwest のみ（API固有型は内部定義）
- `realestate-telemetry` → tracing + metrics（既存）

---

## Tasks

- [ ] Task 1: `lib/geo-math` 実装 — `compute_cagr`, `round_dp`, `bbox_area_deg2` + テスト → Verify: `cargo test -p realestate-geo-math` 通過、compute_score.rs と get_trend.rs から CAGR/round 呼び出しを置換、既存27テスト維持
- [ ] Task 2: `lib/db` 実装 — `DbError`, `bind_bbox`, `bind_coord`, `to_geo_feature`, `create_pool` → Verify: `cargo test -p realestate-db` 通過、全 pg_*_repository.rs から `map_db_err` 削除・`bind_bbox`/`bind_coord` 利用に置換
- [ ] Task 3: `lib/axum-support` 実装 — `ApiError<E>` ジェネリックエラー + `FeatureCollectionDto` 移行 → Verify: `cargo test -p realestate-axum-support` 通過、handler/error.rs と handler/response.rs を薄くラッパーに置換
- [ ] Task 4: `lib/mlit-client` 実装 — `ReinfolibClient` + `KsjClient` スケルトン + `MlitError` → Verify: `cargo test -p mlit-client` 通過（wiremock モックテスト）
- [ ] Task 5: realestate-api 統合 — workspace Cargo.toml 更新、全 infra/usecase/handler から lib クレート利用に移行 → Verify: `cargo clippy --workspace -- -D warnings && cargo test --workspace` 全通過

## Done When

- [ ] `cargo test --workspace` で全テスト通過（既存27 + 新規lib側テスト）
- [ ] `cargo clippy --workspace -- -D warnings` でゼロ警告
- [ ] `map_db_err` のコピーが全リポジトリから消滅（1箇所に集約）
- [ ] CAGR計算が `realestate-geo-math::finance::compute_cagr` に統一
- [ ] 丸め処理が `realestate-geo-math::rounding::round_dp` に統一

## Notes

- Task 1（geo-math）は依存ゼロのため最も安全に着手可能
- Task 2（db）は infra 層への影響が最大 → 慎重にテスト
- Task 4（mlit-client）は Phase 2 向けスケルトンのため後回し可
- domain/ 層は `realestate-api` 内に残す（ビジネスルールはアプリ固有）

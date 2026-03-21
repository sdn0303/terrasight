# 共通ライブラリ抽出計画 Phase 2

## Goal

既存API実装のlib配下に切り出せる共通機能（PostGIS spatial bind helpers、APIサーバーミドルウェア、reinfolib APIクライアント実装）を整理し、SQL linting基盤と合わせて実装する。

---

## Tasks

- [x] Task 1: `lib/db` — spatial bind helpers 実装 → `bind_bbox`, `bind_coord` のフルジェネリックバインドヘルパー + テスト → Verify: `cargo test -p realestate-db` 通過、infra層で利用に置換
- [x] Task 2: `lib/api-core` — ミドルウェア追加 → rate limiting (tower-governor), request ID, response time ミドルウェア → Verify: `cargo test -p realestate-api-core` 通過、main.rs で利用
- [x] Task 3: `lib/mlit-client` — reinfolib endpoint 実装 → tile座標変換 + 7エンドポイントメソッド + 指数バックオフリトライ + wiremockテスト → Verify: `cargo test -p mlit-client` 通過
- [x] Task 4: sqruff SQL lint + lefthook 統合 → sqruff install, .sqruff config, lefthook.yml にSQL lint/fix追加 → Verify: `sqruff lint migrations/` 通過
- [x] Task 5: 統合検証 → `cargo clippy --workspace -- -D warnings && cargo test --workspace` 全通過

## Done When

- [x] `cargo test --workspace` 全通過 (74 unit + 18 doc tests)
- [x] `cargo clippy --workspace -- -D warnings` ゼロ警告
- [x] `bind_bbox`/`bind_coord` が infra 層から利用されている
- [~] rate limiting ミドルウェアが main.rs に組み込まれている (ライブラリ実装済み、ルート別設定は別タスク)
- [x] reinfolib の7エンドポイントが型安全にラップされている
- [x] `sqruff lint migrations/` がゼロ違反
- [x] lefthook pre-commit に SQL lint が追加されている

---

## Task 1 詳細: `lib/db` spatial bind helpers

### 新規ファイル: `lib/db/src/spatial.rs`

```rust
/// PostGIS ST_MakeEnvelope のパラメータを正しい順序でバインドする。
/// 4パラメータ: west (xmin), south (ymin), east (xmax), north (ymax), SRID=4326
pub fn bind_bbox<'q, O>(
    query: QueryAs<'q, Postgres, O, PgArguments>,
    west: f64, south: f64, east: f64, north: f64,
) -> QueryAs<'q, Postgres, O, PgArguments>
where O: Send + Unpin

/// PostGIS ST_MakePoint のパラメータを正しい順序でバインドする。
/// 2パラメータ: longitude (x), latitude (y)
pub fn bind_coord<'q, O>(
    query: QueryAs<'q, Postgres, O, PgArguments>,
    lng: f64, lat: f64,
) -> QueryAs<'q, Postgres, O, PgArguments>
where O: Send + Unpin
```

### 変更対象: infra 層の4リポジトリ

- `pg_area_repository.rs` — 6箇所の bbox バインドを `bind_bbox` に置換
- `pg_stats_repository.rs` — 5箇所の bbox バインドを `bind_bbox` に置換
- `pg_score_repository.rs` — 5箇所の coord バインドを `bind_coord` に置換
- `pg_trend_repository.rs` — 2箇所の coord バインドを `bind_coord` に置換

### テスト

- `bind_bbox` の型推論テスト（sqlx::query_as でコンパイル通過確認）
- `bind_coord` の型推論テスト
- パラメータ順序の正当性テスト（コンパイル時検証）

---

## Task 2 詳細: `lib/api-core` ミドルウェア

### 新規ファイル: `lib/api-core/src/middleware.rs`

```rust
pub mod rate_limit;   // tower-governor ベース
pub mod request_id;   // X-Request-Id 注入
pub mod response_time; // X-Response-Time 計測
```

#### `rate_limit.rs`

```rust
/// エンドポイントごとのレート制限設定
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
}

/// tower-governor の GovernorLayer を構築する
pub fn rate_limit_layer(config: &RateLimitConfig) -> GovernorLayer<...>
```

#### `request_id.rs`

```rust
/// リクエストごとにユニークな UUID を X-Request-Id ヘッダーに注入する Layer
pub fn request_id_layer() -> SetRequestIdLayer<MakeRequestUuid>
```

#### `response_time.rs`

```rust
/// レスポンスに X-Response-Time ヘッダー（ms単位）を追加する Layer
pub fn response_time_layer() -> impl Layer<S>
```

### 依存関係追加 (Cargo.toml)

```toml
tower-governor = "0.4"
tower-http = { version = "0.6", features = ["request-id", "util"] }
uuid = { version = "1", features = ["v4"] }
```

### main.rs 変更

```rust
// Before (current)
.layer(CorsLayer::permissive())

// After
.layer(realestate_api_core::middleware::response_time_layer())
.layer(realestate_api_core::middleware::request_id_layer())
.layer(CorsLayer::permissive())
// rate_limit は個別ルートに適用
```

---

## Task 3 詳細: `lib/mlit-client` reinfolib 実装

### 新規モジュール: `lib/mlit-client/src/tile.rs`

```rust
/// BBox → XYZ タイル座標変換
pub struct TileCoord { pub z: u8, pub x: u32, pub y: u32 }

/// bbox をカバーする全タイル座標を返す
pub fn bbox_to_tiles(west: f64, south: f64, east: f64, north: f64, z: u8) -> Vec<TileCoord>

/// 緯度経度 → タイルX座標
fn lng_to_tile_x(lng: f64, z: u8) -> u32

/// 緯度 → タイルY座標
fn lat_to_tile_y(lat: f64, z: u8) -> u32
```

### reinfolib.rs 追加メソッド（7エンドポイント）

```rust
impl ReinfolibClient {
    /// XPT001: 取引価格ポイント
    pub async fn get_transaction_prices(
        &self, west: f64, south: f64, east: f64, north: f64,
        from: &str, to: &str,
    ) -> Result<Vec<serde_json::Value>, MlitError>;

    /// XPT002: 地価公示ポイント
    pub async fn get_land_prices(
        &self, west: f64, south: f64, east: f64, north: f64,
        year: u16,
    ) -> Result<Vec<serde_json::Value>, MlitError>;

    /// XIT001: 取引価格情報（非タイル、地域指定）
    pub async fn get_transaction_data(
        &self, year: u16, quarter: u8, area: &str,
    ) -> Result<Vec<serde_json::Value>, MlitError>;

    /// XKT002: 用途地域
    pub async fn get_zoning(
        &self, west: f64, south: f64, east: f64, north: f64,
    ) -> Result<Vec<serde_json::Value>, MlitError>;

    /// XKT006: 学校
    pub async fn get_schools(
        &self, west: f64, south: f64, east: f64, north: f64,
    ) -> Result<Vec<serde_json::Value>, MlitError>;

    /// XKT010: 医療機関
    pub async fn get_medical(
        &self, west: f64, south: f64, east: f64, north: f64,
    ) -> Result<Vec<serde_json::Value>, MlitError>;

    /// XKT016: 災害危険区域
    pub async fn get_hazard_areas(
        &self, west: f64, south: f64, east: f64, north: f64,
    ) -> Result<Vec<serde_json::Value>, MlitError>;
}
```

### 内部ヘルパー

```rust
impl ReinfolibClient {
    /// タイルベースエンドポイントの共通fetch + マージロジック
    async fn fetch_tiles(
        &self,
        endpoint: &str,
        tiles: &[TileCoord],
        extra_params: &[(&str, &str)],
    ) -> Result<Vec<serde_json::Value>, MlitError>;

    /// 指数バックオフリトライ（3回、1s/2s/4s）
    async fn request_with_retry(
        &self,
        url: &str,
        params: &[(&str, String)],
    ) -> Result<reqwest::Response, MlitError>;
}
```

### テスト（wiremock）

- `tile.rs`: bbox→tiles変換の正当性テスト（東京駅周辺、z=14）
- `reinfolib.rs`: wiremock で200/429/500レスポンスをモック
  - 正常系: GeoJSON features のデシリアライズ
  - リトライ: 429→429→200 で3回目に成功
  - エラー: 500 で `MlitError::Api` 返却

### 依存関係追加 (Cargo.toml)

```toml
[dev-dependencies]
wiremock = "0.6"
tokio = { version = "1", features = ["rt", "macros"] }
```

---

## Task 4 詳細: sqruff SQL lint + lefthook

### インストール

```bash
brew install sqruff
```

### 設定ファイル: `services/backend/.sqruff`

```ini
[sqruff]
dialect = postgres
```

### lefthook.yml 追加

```yaml
pre-commit:
  commands:
    sql-lint:
      root: "services/backend/"
      glob: "*.sql"
      run: sqruff lint migrations/
      fail_text: "SQL lint failed — run `sqruff fix migrations/` to fix"
```

---

## Notes

- Task 1 と Task 4 は独立、並行実行可能
- Task 2 は api-core の Cargo.toml 変更があるため Task 1 完了後
- Task 3 は最大規模、他タスク完了後に集中実装
- reinfolib APIキー未取得のため、全テストは wiremock モック

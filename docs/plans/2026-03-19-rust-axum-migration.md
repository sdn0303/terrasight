# Rust Axum バックエンド移行計画

> **前提**: `docs/plans/2026-03-18-realestate-visualizer.md` の Task 1-11 は Python FastAPI で実装済み。
> このドキュメントはバックエンドを Rust Axum に書き換える計画。

**Goal:** Python FastAPI バックエンド (~200行) を Rust Axum に移行し、同時に SQLite → PostGIS 移行を実施する。フロントエンドの API インターフェースは変更しない。

**Architecture:** Next.js (TypeScript) フロントエンド + **Rust Axum** バックエンドの2サービス構成。Axum が国交省 API のプロキシ兼 PostGIS 空間クエリ層として動作し、GeoJSON 形式でフロントへ配信。

**Tech Stack:**
- Frontend: Next.js 16 + TypeScript + react-map-gl + MapLibre GL JS（変更なし。Phase 2以降でMapbox検討）
- Backend: **Rust (Axum + tokio + sqlx + geo + geozero + serde)**
- DB: **PostgreSQL + PostGIS**（SQLite から移行）
- Map: CARTO Dark Matter basemap（変更なし）

---

## 判断メモ

### Python FastAPI → Rust Axum 移行理由

1. **全国 GeoJSON（数GB）の bbox フィルタリング**: Python の GIL 制約 + json.loads のオーバーヘッドがボトルネック。Rust の serde_json + geozero でゼロコピーパース可能
2. **投資スコア空間計算**: 半径検索 + ポリゴン重畳率計算がホットパス。geo crate でネイティブ速度
3. **メモリ効率**: 全国データをインメモリ処理する場面で Python の 5-10 倍効率的
4. **デプロイ**: シングルバイナリ ~10MB（Python + deps = 数百MB）
5. **同時リクエスト**: tokio のワークスティーリングが GIL なしで活きる

### 移行戦略

- フロントエンドの API インターフェース（パス + レスポンス JSON 構造）は**一切変更しない**
- Python バックエンドは Week 1 まで稼働、Week 2 で Rust に完全切り替え
- docker-compose.yml の `backend` サービスを差し替え

---

## Task 1: Rust プロジェクト初期化

**Files:**
- Create: `services/backend/Cargo.toml`
- Create: `services/backend/src/main.rs`
- Create: `services/backend/src/routes/mod.rs`
- Create: `services/backend/src/routes/health.rs`
- Create: `services/backend/.env.example`

**Step 1: Cargo.toml**

```toml
[package]
name = "realestate-api"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "json"] }
reqwest = { version = "0.12", features = ["json"] }
geo = "0.29"
geo-types = "0.7"
geozero = { version = "0.14", features = ["with-geojson", "with-postgis-sqlx"] }
geojson = "0.24"
tower-http = { version = "0.6", features = ["cors", "compression-gzip"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
thiserror = "2"
anyhow = "1"
```

**Step 2: main.rs スケルトン**

```rust
use axum::{Router, routing::get};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;
use tower_http::compression::CompressionLayer;
use std::net::SocketAddr;

mod routes;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub reinfolib_key: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/realestate".into());
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await?;

    let state = AppState {
        db: pool,
        reinfolib_key: std::env::var("REINFOLIB_API_KEY").ok(),
    };

    let app = Router::new()
        .route("/api/health", get(routes::health::health))
        .route("/api/area-data", get(routes::area_data::get_area_data))
        .route("/api/score", get(routes::score::get_score))
        .route("/api/stats", get(routes::stats::get_stats))
        .route("/api/trend", get(routes::trend::get_trend))
        .with_state(state)
        .layer(CorsLayer::permissive())  // 本番では制限する
        .layer(CompressionLayer::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

**Step 3: .env.example**

```
DATABASE_URL=postgres://user:pass@localhost:5432/realestate
REINFOLIB_API_KEY=
ESTAT_APP_ID=
RUST_LOG=info
```

**Step 4: ビルド確認**

```bash
cd backend-rs
cargo build
cargo run  # → http://localhost:8000/api/health
```

**Step 5: コミット**

```bash
git add services/backend/
git commit -m "feat: Rust Axum backend scaffold with PostGIS connection"
```

---

## Task 2: PostGIS セットアップ + データインポート

**Files:**
- Create: `services/backend/migrations/001_init.sql`
- Create: `scripts/import_geojson.sh`

**Step 1: マイグレーション SQL**

```sql
CREATE EXTENSION IF NOT EXISTS postgis;

-- 地価公示（国土数値情報 L01）
CREATE TABLE land_prices (
    id SERIAL PRIMARY KEY,
    price_per_sqm INTEGER,
    address TEXT,
    land_use TEXT,
    year INTEGER,
    geom GEOMETRY(Point, 4326) NOT NULL
);
CREATE INDEX idx_land_prices_geom ON land_prices USING GIST(geom);

-- 用途地域（国土数値情報 A29）
CREATE TABLE zoning (
    id SERIAL PRIMARY KEY,
    zone_type TEXT,
    zone_code TEXT,
    floor_area_ratio REAL,
    building_coverage REAL,
    geom GEOMETRY(MultiPolygon, 4326) NOT NULL
);
CREATE INDEX idx_zoning_geom ON zoning USING GIST(geom);

-- 洪水浸水想定区域（国土数値情報 A31）
CREATE TABLE flood_risk (
    id SERIAL PRIMARY KEY,
    depth_rank TEXT,
    river_name TEXT,
    geom GEOMETRY(MultiPolygon, 4326) NOT NULL
);
CREATE INDEX idx_flood_risk_geom ON flood_risk USING GIST(geom);

-- 急傾斜地（国土数値情報 A47）
CREATE TABLE steep_slope (
    id SERIAL PRIMARY KEY,
    area_name TEXT,
    geom GEOMETRY(MultiPolygon, 4326) NOT NULL
);
CREATE INDEX idx_steep_slope_geom ON steep_slope USING GIST(geom);

-- 学校（国土数値情報 P29）
CREATE TABLE schools (
    id SERIAL PRIMARY KEY,
    name TEXT,
    school_type TEXT,
    geom GEOMETRY(Point, 4326) NOT NULL
);
CREATE INDEX idx_schools_geom ON schools USING GIST(geom);

-- 医療機関（国土数値情報 P04）
CREATE TABLE medical_facilities (
    id SERIAL PRIMARY KEY,
    name TEXT,
    facility_type TEXT,
    bed_count INTEGER,
    geom GEOMETRY(Point, 4326) NOT NULL
);
CREATE INDEX idx_medical_geom ON medical_facilities USING GIST(geom);
```

**Step 2: sqlx マイグレーション実行**

```bash
sqlx migrate run
```

**Step 3: GeoJSON インポートスクリプト（ogr2ogr）**

```bash
#!/bin/bash
# scripts/import_geojson.sh
DB_URL=${DATABASE_URL:-"postgresql://localhost/realestate"}

# 地価公示（5年分 — Sparkline用）
for YEAR in 20 21 22 23 24; do
  ogr2ogr -f "PostgreSQL" "$DB_URL" "data/geojson/L01-${YEAR}_13.geojson" \
    -nln land_prices -append -a_srs EPSG:4326
done

# 用途地域
ogr2ogr -f "PostgreSQL" "$DB_URL" data/geojson/A29-11_13.geojson \
  -nln zoning -append -a_srs EPSG:4326

# 洪水浸水想定区域
ogr2ogr -f "PostgreSQL" "$DB_URL" data/geojson/A31-12_13.geojson \
  -nln flood_risk -append -a_srs EPSG:4326

# 急傾斜地
ogr2ogr -f "PostgreSQL" "$DB_URL" data/geojson/A47-20_13.geojson \
  -nln steep_slope -append -a_srs EPSG:4326

# 学校
ogr2ogr -f "PostgreSQL" "$DB_URL" data/geojson/P29-21_13.geojson \
  -nln schools -append -a_srs EPSG:4326

# 医療機関
ogr2ogr -f "PostgreSQL" "$DB_URL" data/geojson/P04-14_13.geojson \
  -nln medical_facilities -append -a_srs EPSG:4326

echo "Import complete. Verifying row counts..."
psql "$DB_URL" -c "SELECT 'land_prices' as tbl, count(*) FROM land_prices
  UNION ALL SELECT 'zoning', count(*) FROM zoning
  UNION ALL SELECT 'flood_risk', count(*) FROM flood_risk
  UNION ALL SELECT 'schools', count(*) FROM schools
  UNION ALL SELECT 'medical_facilities', count(*) FROM medical_facilities;"
```

**Step 4: seed データ（開発用最小セット）**

> PostGIS にデータ投入される前の開発段階でフロントエンド開発者が動作確認できるよう、東京駅周辺の最小サンプルデータを用意する。

```bash
# scripts/seed_dev_data.sql — 開発用最小データ（東京駅周辺）
INSERT INTO land_prices (price_per_sqm, address, land_use, year, geom)
VALUES
  (1200000, '千代田区丸の内1', '商業', 2024, ST_SetSRID(ST_MakePoint(139.7671, 35.6812), 4326)),
  (800000, '中央区銀座4', '商業', 2024, ST_SetSRID(ST_MakePoint(139.7649, 35.6717), 4326)),
  (600000, '港区新橋2', '商業', 2024, ST_SetSRID(ST_MakePoint(139.7586, 35.6660), 4326));
-- 他テーブルも同様に5-10行ずつ
```

**Step 5: コミット**

```bash
git commit -m "feat: PostGIS schema, import pipeline, and dev seed data"
```

---

## Task 3: /api/area-data エンドポイント（bbox クエリ）

> **注意**: エンドポイントパスは既存FEとの互換性のため **`/api/area-data`** を維持する（`/api/layers` にリネームしない）。

**Files:**
- Create: `services/backend/src/routes/area_data.rs`
- Create: `services/backend/src/models/geojson.rs`

**Step 1: bbox パラメータから PostGIS クエリ（パラメータバインド使用）**

> **CRITICAL: SQLインジェクション防止** — bbox値は必ず `$1`, `$2` バインドパラメータで渡す。`format!()` でSQL文字列に埋め込まないこと。

```rust
use axum::{extract::{Query, State}, Json};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AreaDataQuery {
    south: f64,
    west: f64,
    north: f64,
    east: f64,
    layers: String,  // "landprice,zoning,flood"
}

pub async fn get_area_data(
    State(state): State<AppState>,
    Query(q): Query<AreaDataQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    // bbox面積制限（DoS防止）: 緯度経度で約0.5度四方まで
    if (q.north - q.south).abs() > 0.5 || (q.east - q.west).abs() > 0.5 {
        return Err(AppError::BadRequest("bbox too large".into()));
    }

    let layer_list: Vec<&str> = q.layers.split(',').map(|s| s.trim()).collect();
    let mut result = serde_json::Map::new();

    for layer in layer_list {
        let fc = match layer {
            "landprice" => query_land_prices(&state.db, q.west, q.south, q.east, q.north).await?,
            "zoning" => query_zoning(&state.db, q.west, q.south, q.east, q.north).await?,
            "flood" => query_flood(&state.db, q.west, q.south, q.east, q.north).await?,
            "steep_slope" => query_steep_slope(&state.db, q.west, q.south, q.east, q.north).await?,
            "schools" => query_schools(&state.db, q.west, q.south, q.east, q.north).await?,
            "medical" => query_medical(&state.db, q.west, q.south, q.east, q.north).await?,
            _ => continue,
        };
        result.insert(layer.to_string(), fc);
    }

    Ok(Json(serde_json::Value::Object(result)))
}
```

**Step 2: 各テーブルのクエリ関数（バインドパラメータ使用）**

```rust
async fn query_land_prices(
    pool: &PgPool, west: f64, south: f64, east: f64, north: f64,
) -> Result<serde_json::Value> {
    let rows = sqlx::query_as::<_, LandPriceRow>(r#"
        SELECT id, price_per_sqm, address, land_use,
               ST_AsGeoJSON(geom)::jsonb as geometry
        FROM land_prices
        WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
        "#, bbox)
    )
    "#)
    .bind(west).bind(south).bind(east).bind(north)
    .fetch_all(pool)
    .await?;

    // → GeoJSON FeatureCollection に変換
    Ok(to_feature_collection(rows))
}
```

**Step 3: フロントエンドとの互換性確認**

- エンドポイント: `/api/area-data`（Python版と同一パス）
- レスポンス形式: `{ "landprice": { "type": "FeatureCollection", ... }, "zoning": { ... } }`
- Python 版と同一の JSON 構造を維持
- **`useMapData.ts` の変更は不要**

**Step 4: コミット**

```bash
git commit -m "feat: /api/area-data with PostGIS spatial queries (parameterized)"
```

---

## Task 4: /api/score エンドポイント（投資スコアリング）

**Files:**
- Create: `services/backend/src/routes/score.rs`
- Create: `services/backend/src/services/scoring.rs`

**概要:**
- 入力: `lat`, `lng`
- 出力: `{ score: 72, components: { trend: 18, risk: 22, access: 15, yield: 17 }, details: { ... } }`
- PostGIS の ST_DWithin で半径検索、geo crate で面積計算

**Step 1: スコアリングエンジン（geo crate 活用）**

```rust
pub struct InvestmentScore {
    pub total: f64,        // 0-100
    pub trend: f64,        // 0-25: 地価トレンド
    pub risk: f64,         // 0-25: 防災リスク（反転）
    pub access: f64,       // 0-25: 施設アクセス
    pub yield_score: f64,  // 0-25: 利回りポテンシャル
}
```

---

## Task 5: /api/stats + /api/trend エンドポイント

- `/api/stats?bbox=...` → ビューポート内の集計統計（平均地価、物件数、リスク分布）
- `/api/trend?lat=...&lng=...&years=10` → 最寄り地点の年度別地価推移

---

## Task 6: reinfolib API プロキシ（reqwest）

- Python の `fetch_reinfolib()` を reqwest で移植
- 指数バックオフ付きリトライ（tower::retry or 手動）
- 取得した GeoJSON を PostGIS にキャッシュ

---

## Task 7: Docker + docker-compose 更新

**Files:**
- Create: `services/backend/Dockerfile`
- Modify: `docker-compose.yml`

```dockerfile
# services/backend/Dockerfile
FROM rust:1.82 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/realestate-api /usr/local/bin/
CMD ["realestate-api"]
```

docker-compose.yml に PostgreSQL + PostGIS サービスを追加:

```yaml
services:
  db:
    image: postgis/postgis:16-3.4
    environment:
      POSTGRES_DB: realestate
      POSTGRES_USER: app
      POSTGRES_PASSWORD: ${DB_PASSWORD:-devpass}
    volumes:
      - pgdata:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  backend:
    build: ./services/backend
    depends_on:
      db:
        condition: service_healthy
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8000/api/health"]
      interval: 30s
      timeout: 5s
      retries: 3
    environment:
      DATABASE_URL: postgres://app:${DB_PASSWORD:-devpass}@db:5432/realestate
      REINFOLIB_API_KEY: ${REINFOLIB_API_KEY:-}
    ports:
      - "8000:8000"

  frontend:
    build: ./services/frontend
    depends_on:
      - backend
    environment:
      NEXT_PUBLIC_API_URL: http://localhost:8000
    ports:
      - "3000:3000"

volumes:
  pgdata:
```

---

## Task 8: Python バックエンド撤去

- 旧 `backend/`（Python）と `frontend/` は削除または `_legacy/` に退避
- 新構造は `services/backend/`（Rust）と `services/frontend/`（Next.js）
- `.claude/launch.json` 更新
- `docker-compose.yml` の build パスを `./services/backend`, `./services/frontend` に更新

---

## 移行チェックリスト

### バックエンド（Python → Rust）

| 項目 | Python 版 | Rust 版 | 状態 |
|------|----------|---------|------|
| GET /api/health | ✓ | □ | |
| GET /api/area-data | ✓ | □ | パス維持（リネームしない） |
| GET /api/score | — | □ | 新規 |
| GET /api/stats | — | □ | 新規 |
| GET /api/trend | — | □ | 新規 |
| DEMO_MODE (モックデータ) | ✓ | □ | PostGIS + 実データで不要に |
| reinfolib API プロキシ | ✓ | □ | |
| CORS | ✓ | □ | tower-http |
| Gzip圧縮 | ✓ | □ | tower-http compression |
| エラーハンドリング | 部分的 | □ | thiserror で統一 |
| 構造化ログ | ✗ | □ | tracing で新規 |

### フロントエンド（変更なし — MapLibre維持）

Phase 1ではフロントエンドの地図ライブラリ変更なし。Mapbox GL JSへの切替はPhase 2以降で検討。

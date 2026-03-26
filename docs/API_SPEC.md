# API_SPEC.md — Rust Axum Backend REST API 仕様書

> Version: 1.0.0 | Updated: 2026-03-20
> Runtime: Rust Axum 0.8 + tokio + sqlx + PostGIS
> Base URL: `http://localhost:8000`

---

## 1. 共通仕様

### 1.1 リクエスト/レスポンス形式
- Content-Type: `application/json`
- GeoJSON レスポンスは RFC 7946 準拠
- 文字エンコーディング: UTF-8
- 圧縮: gzip（tower-http CompressionLayer）

### 1.2 エラーレスポンス

```json
{
  "error": {
    "code": "BBOX_TOO_LARGE",
    "message": "Bounding box exceeds maximum allowed area (0.5 degrees)"
  }
}
```

| HTTP Status | コード | 説明 |
|------------|--------|------|
| 400 | `INVALID_PARAMS` | パラメータ不正（型、範囲） |
| 400 | `BBOX_TOO_LARGE` | bbox面積が0.5度四方を超過 |
| 404 | `NOT_FOUND` | リソースが見つからない |
| 429 | `RATE_LIMITED` | レート制限超過 |
| 500 | `INTERNAL_ERROR` | サーバー内部エラー |
| 503 | `DB_UNAVAILABLE` | PostGIS接続不可 |

### 1.3 共通ヘッダー

**レスポンス:**
```
Content-Type: application/json; charset=utf-8
Content-Encoding: gzip
X-Request-Id: {uuid}
X-Response-Time: {ms}
```

### 1.4 CORS
- Phase 1: `CorsLayer::permissive()` → 環境変数 `ALLOWED_ORIGINS` で制限に移行
- Phase 2: 認証付きエンドポイントは credentials 対応

### 1.5 レート制限（Phase 1 P1）
- tower-governor による IP ベースレート制限
- `/api/area-data`: 30 req/min
- `/api/score`: 60 req/min
- `/api/stats`, `/api/trend`: 60 req/min

---

## 2. エンドポイント一覧

### Phase 1

| Method | Path | 概要 | 認証 |
|--------|------|------|------|
| GET | `/api/health` | ヘルスチェック | 不要 |
| GET | `/api/area-data` | レイヤーデータ取得（bbox） | 不要 |
| GET | `/api/score` | 投資スコア算出 | 不要 |
| GET | `/api/stats` | エリア統計集計 | 不要 |
| GET | `/api/trend` | 地価推移データ | 不要 |

### Phase 2（SaaS化）

| Method | Path | 概要 | 認証 |
|--------|------|------|------|
| POST | `/api/auth/register` | ユーザー登録 | 不要 |
| POST | `/api/auth/login` | ログイン | 不要 |
| POST | `/api/auth/refresh` | トークン更新 | Bearer |
| GET | `/api/watchlist` | ウォッチリスト取得 | Bearer |
| POST | `/api/watchlist` | ウォッチリスト条件追加 | Bearer |
| DELETE | `/api/watchlist/:id` | ウォッチリスト条件削除 | Bearer |

---

## 3. エンドポイント詳細

### 3.1 GET /api/health

ヘルスチェック。DB接続状態とAPIキー設定状態を返す。

**Response 200:**
```json
{
  "status": "ok",
  "db_connected": true,
  "reinfolib_key_set": false,
  "version": "0.1.0",
  "uptime_seconds": 3600
}
```

**Response 503 (DB接続不可):**
```json
{
  "status": "degraded",
  "db_connected": false,
  "error": { "code": "DB_UNAVAILABLE", "message": "Cannot connect to PostGIS" }
}
```

---

### 3.2 GET /api/area-data

ビューポート（bbox）内の地理空間レイヤーデータを取得。

> **重要**: このエンドポイントパスは既存FE互換のため維持。リネーム禁止。

**Query Parameters:**

| Param | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| south | f64 | Yes | bbox南端緯度 | 35.65 |
| west | f64 | Yes | bbox西端経度 | 139.70 |
| north | f64 | Yes | bbox北端緯度 | 35.70 |
| east | f64 | Yes | bbox東端経度 | 139.80 |
| layers | string | Yes | カンマ区切りのレイヤーID | landprice,zoning |

**有効なレイヤーID:**
- `landprice` — 地価公示（Point）
- `zoning` — 用途地域（MultiPolygon）
- `flood` — 洪水浸水想定区域（MultiPolygon）
- `steep_slope` — 急傾斜地崩壊危険区域（MultiPolygon）
- `schools` — 学校（Point）
- `medical` — 医療機関（Point）

**バリデーション:**
- bbox面積制限: `|north - south| <= 0.5` かつ `|east - west| <= 0.5`
- south < north, west < east
- 緯度: -90 〜 90、経度: -180 〜 180

**Response 200:**
```json
{
  "landprice": {
    "type": "FeatureCollection",
    "features": [
      {
        "type": "Feature",
        "geometry": {
          "type": "Point",
          "coordinates": [139.7671, 35.6812]
        },
        "properties": {
          "id": 1,
          "price_per_sqm": 1200000,
          "address": "千代田区丸の内1",
          "land_use": "商業",
          "year": 2024
        }
      }
    ]
  },
  "zoning": {
    "type": "FeatureCollection",
    "features": [
      {
        "type": "Feature",
        "geometry": {
          "type": "MultiPolygon",
          "coordinates": [[[[139.76, 35.68], [139.77, 35.68], [139.77, 35.69], [139.76, 35.69], [139.76, 35.68]]]]
        },
        "properties": {
          "id": 1,
          "zone_type": "商業地域",
          "zone_code": "09",
          "floor_area_ratio": 8.0,
          "building_coverage": 0.8
        }
      }
    ]
  }
}
```

**PostGIS クエリ（各レイヤー共通パターン）:**
```sql
SELECT id, price_per_sqm, address, land_use, year,
       ST_AsGeoJSON(geom)::jsonb AS geometry
FROM land_prices
WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
-- $1=west, $2=south, $3=east, $4=north
-- CRITICAL: パラメータバインド必須。format!()禁止
```

---

### 3.3 GET /api/score

指定座標の投資スコアを算出。

**Query Parameters:**

| Param | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| lat | f64 | Yes | 緯度 | 35.6812 |
| lng | f64 | Yes | 経度 | 139.7671 |

**Response 200:**
```json
{
  "score": 72,
  "components": {
    "trend": {
      "value": 18,
      "max": 25,
      "detail": {
        "cagr_5y": 0.032,
        "direction": "up",
        "latest_price": 1200000,
        "price_5y_ago": 1020000
      }
    },
    "risk": {
      "value": 22,
      "max": 25,
      "detail": {
        "flood_overlap": 0.0,
        "liquefaction_overlap": 0.05,
        "steep_slope_nearby": false,
        "composite_risk": 0.12
      }
    },
    "access": {
      "value": 15,
      "max": 25,
      "detail": {
        "schools_1km": 3,
        "medical_1km": 5,
        "nearest_school_m": 450,
        "nearest_medical_m": 200
      }
    },
    "yield_potential": {
      "value": 17,
      "max": 25,
      "detail": {
        "avg_transaction_price": 950000,
        "land_price": 1200000,
        "estimated_yield": 0.048
      }
    }
  },
  "metadata": {
    "calculated_at": "2026-03-20T10:30:00Z",
    "data_freshness": "2024",
    "disclaimer": "本スコアは参考値です。投資判断は自己責任で行ってください。"
  }
}
```

**計算ロジック:**

```
trend (0-25):
  CAGR = (latest / 5y_ago)^(1/5) - 1
  score = clamp(CAGR * 500, 0, 25)  // CAGR 5% → 25点

risk (0-25):  // 反転: 低リスク = 高スコア
  composite = flood_overlap * 0.4 + liquefaction_overlap * 0.4 + steep_slope * 0.2
  score = 25 * (1 - composite)

access (0-25):
  school_score = min(schools_1km / 3, 1.0) * 10
  medical_score = min(medical_1km / 5, 1.0) * 10
  distance_score = max(0, 5 - nearest_school_m / 200)  // 近いほど高い
  score = clamp(school_score + medical_score + distance_score, 0, 25)

yield_potential (0-25):
  yield = avg_transaction_price / land_price
  score = clamp(yield * 500, 0, 25)  // yield 5% → 25点
```

**PostGIS クエリ（コンポーネント別）:**

```sql
-- trend: 最寄り地点の5年間地価
SELECT year, price_per_sqm
FROM land_prices
WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000)
ORDER BY ST_Distance(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography)
LIMIT 1;
-- 同一地点の複数年度を year でグループ

-- risk: 半径500m内のリスクポリゴン重畳率
SELECT COALESCE(
  SUM(ST_Area(ST_Intersection(
    ST_Buffer(ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)::geometry, geom
  ))) / ST_Area(ST_Buffer(ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500)::geometry),
  0.0
) FROM flood_risk
WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 500);

-- access: 半径1km内の施設数
SELECT count(*) FROM schools
WHERE ST_DWithin(geom::geography, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, 1000);
```

---

### 3.4 GET /api/stats

ビューポート内のエリア統計を集計。

**Query Parameters:**

| Param | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| south | f64 | Yes | bbox南端緯度 | 35.65 |
| west | f64 | Yes | bbox西端経度 | 139.70 |
| north | f64 | Yes | bbox北端緯度 | 35.70 |
| east | f64 | Yes | bbox東端経度 | 139.80 |

**Response 200:**
```json
{
  "land_price": {
    "avg_per_sqm": 850000,
    "median_per_sqm": 720000,
    "min_per_sqm": 320000,
    "max_per_sqm": 3200000,
    "count": 45
  },
  "risk": {
    "flood_area_ratio": 0.15,
    "steep_slope_area_ratio": 0.02,
    "avg_composite_risk": 0.18
  },
  "facilities": {
    "schools": 12,
    "medical": 28
  },
  "zoning_distribution": {
    "商業地域": 0.35,
    "第一種住居地域": 0.25,
    "第一種中高層住居専用地域": 0.20,
    "その他": 0.20
  }
}
```

**PostGIS クエリ:**
```sql
SELECT
  AVG(price_per_sqm) as avg_price,
  PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY price_per_sqm) as median_price,
  MIN(price_per_sqm) as min_price,
  MAX(price_per_sqm) as max_price,
  COUNT(*) as count
FROM land_prices
WHERE ST_Intersects(geom, ST_MakeEnvelope($1, $2, $3, $4, 4326))
  AND year = (SELECT MAX(year) FROM land_prices);
```

---

### 3.5 GET /api/trend

指定地点の最寄り観測点における地価推移を返す。

**Query Parameters:**

| Param | Type | Required | Description | Default | Example |
|-------|------|----------|-------------|---------|---------|
| lat | f64 | Yes | 緯度 | — | 35.6812 |
| lng | f64 | Yes | 経度 | — | 139.7671 |
| years | i32 | No | 取得年数 | 5 | 10 |

**Response 200:**
```json
{
  "location": {
    "address": "千代田区丸の内1",
    "distance_m": 120
  },
  "data": [
    { "year": 2020, "price_per_sqm": 1020000 },
    { "year": 2021, "price_per_sqm": 1050000 },
    { "year": 2022, "price_per_sqm": 1100000 },
    { "year": 2023, "price_per_sqm": 1150000 },
    { "year": 2024, "price_per_sqm": 1200000 }
  ],
  "cagr": 0.032,
  "direction": "up"
}
```

---

## 4. Rust 型定義（参考）

```rust
// AppState
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub reinfolib_key: Option<String>,
}

// Error型
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Not found")]
    NotFound,
    #[error("Rate limited")]
    RateLimited,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response { /* ... */ }
}

// 投資スコア
#[derive(Serialize)]
pub struct InvestmentScore {
    pub score: f64,
    pub components: ScoreComponents,
    pub metadata: ScoreMetadata,
}

#[derive(Serialize)]
pub struct ScoreComponents {
    pub trend: ScoreDetail,
    pub risk: ScoreDetail,
    pub access: ScoreDetail,
    pub yield_potential: ScoreDetail,
}
```

---

## 5. 外部API連携（Phase 2）

### 5.1 reinfolib API

```
Base URL: https://www.reinfolib.mlit.go.jp/ex-api/external/{ENDPOINT}
Header: Ocp-Apim-Subscription-Key: {REINFOLIB_API_KEY}
Format: GeoJSON (response_format=geojson)
Tile params: z={zoom}&x={tile_x}&y={tile_y}
```

**Rust実装パターン（reqwest + 指数バックオフ）:**
```rust
async fn fetch_reinfolib(client: &reqwest::Client, endpoint: &str, params: &[(&str, &str)]) -> Result<serde_json::Value> {
    let url = format!("https://www.reinfolib.mlit.go.jp/ex-api/external/{}", endpoint);
    for attempt in 0..3 {
        match client.get(&url)
            .query(params)
            .header("Ocp-Apim-Subscription-Key", &api_key)
            .send().await
        {
            Ok(resp) if resp.status() == 429 => {
                tokio::time::sleep(Duration::from_millis(1000 * 2u64.pow(attempt))).await;
                continue;
            }
            Ok(resp) => return Ok(resp.json().await?),
            Err(e) => return Err(e.into()),
        }
    }
    Err(anyhow!("Max retries exceeded"))
}
```

### 5.2 e-Stat API

```
Base URL: https://api.e-stat.go.jp/rest/3.0/app/json/getStatsData
Param: appId={ESTAT_APP_ID}
Format: JSON
```

---

## 6. データベースマイグレーション

`services/backend/migrations/` に sqlx マイグレーションファイルを配置。

```bash
# マイグレーション実行
sqlx migrate run

# データベースリセット（マイグレーション + 開発用seedデータ投入）
./scripts/commands/db-reset.sh

# 本番データインポート（国土数値情報）
./scripts/commands/db-import.sh
```

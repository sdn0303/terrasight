# API_SPEC.md — Rust Axum Backend REST API 仕様書

> Version: 2.0.0 | Updated: 2026-04-11
> Runtime: Rust Axum + tokio + sqlx + PostGIS
> Base URL: `http://localhost:8000`
>
> **API contract source of truth**: `services/frontend/src/lib/schemas.ts` (Zod).
> Backend DTO と Zod スキーマは必ず一致させる。差異は integration test で assert する。
> 参照: `AGENTS.md §API Contract Rules`, `services/backend/src/handler/response.rs`.

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
- 実際の制限値は `services/backend/src/main.rs` の `rate_limit_rpm` / `rate_limit_burst` 設定を参照
- 環境変数 `RATE_LIMIT_RPM`, `RATE_LIMIT_BURST` で上書き可

---

## 2. エンドポイント一覧

### Phase 1 (current)

| Method | Path | 概要 | 認証 |
|--------|------|------|------|
| GET | `/api/health` | ヘルスチェック | 不要 |
| GET | `/api/area-data` | 複数レイヤーデータ取得（bbox） | 不要 |
| GET | `/api/area-stats` | 行政区コード別の集計統計 | 不要 |
| GET | `/api/v1/land-prices` | 地価公示（単年 + bbox） | 不要 |
| GET | `/api/v1/land-prices/all-years` | 地価公示（時系列 2019〜2024, time machine 用） | 不要 |
| GET | `/api/score` | 投資スコア算出 (TLS 5軸 + 4プリセット) | 不要 |
| GET | `/api/stats` | bbox 統計集計 | 不要 |
| GET | `/api/trend` | 地価推移データ | 不要 |

> 実装: `services/backend/src/lib.rs` の `build_router`.

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

指定座標の Total Location Score (TLS, 0-100) を 5 軸で算出。
選択した重みプリセットに応じて最終スコアが変化する。

**Query Parameters:**

| Param | Type | Required | Default | Description | Example |
|-------|------|----------|---------|-------------|---------|
| lat | f64 | Yes | — | 緯度 | 35.6812 |
| lng | f64 | Yes | — | 経度 | 139.7671 |
| preset | string | No | `balance` | 重みプリセット | `investment` |

**`preset` に受け付ける値:**
- `balance` — デフォルト (disaster 0.25 / terrain 0.15 / livability 0.25 / future 0.15 / price 0.20)
- `investment` — 投資家向け (disaster 0.15 / terrain 0.10 / livability 0.20 / future 0.25 / price 0.30)
- `residential` — 居住者向け (disaster 0.25 / terrain 0.15 / livability 0.35 / future 0.10 / price 0.15)
- `disaster` / `disaster_focus` — 防災重視 (disaster 0.40 / terrain 0.25 / livability 0.20 / future 0.05 / price 0.10)

未知の値は `balance` にフォールバック。
実装: `services/backend/src/handler/request.rs` `CoordQuery::parse_preset`,
`services/backend/src/domain/scoring/tls.rs` `WeightPreset::weights`.

**Response 200** (Zod: `TlsResponse` in `services/frontend/src/lib/schemas.ts`):

```json
{
  "location": { "lat": 35.6812, "lng": 139.7671 },
  "tls": {
    "score": 72.4,
    "grade": "A",
    "label": "Very Good"
  },
  "axes": {
    "disaster": {
      "score": 78.2,
      "weight": 0.25,
      "confidence": 0.85,
      "sub": [
        { "id": "flood",        "score": 80.0, "available": true,  "detail": { "depth_rank": 1 } },
        { "id": "liquefaction", "score": 100.0,"available": false, "detail": { "pl_value": null } },
        { "id": "seismic",      "score": 65.0, "available": true,  "detail": { "prob_30yr": 0.23 } },
        { "id": "tsunami",      "score": 100.0,"available": true,  "detail": { "depth_m": null } },
        { "id": "landslide",    "score": 100.0,"available": true,  "detail": { "steep_nearby": false } }
      ]
    },
    "terrain":    { "score": 70.0, "weight": 0.15, "confidence": 1.0, "sub": [ /* avs30 */ ] },
    "livability": { "score": 82.0, "weight": 0.25, "confidence": 0.67, "sub": [ /* transit, education, medical */ ] },
    "future":     { "score": 58.0, "weight": 0.15, "confidence": 0.67, "sub": [ /* population, price_trend, far */ ] },
    "price":      { "score": 71.0, "weight": 0.20, "confidence": 1.0, "sub": [ /* relative_value, volume */ ] }
  },
  "cross_analysis": {
    "value_discovery": 56.0,
    "demand_signal":   47.6,
    "ground_safety":   54.7
  },
  "metadata": {
    "calculated_at": "2026-04-11T00:00:00Z",
    "weight_preset": "balance",
    "data_freshness": "2024",
    "disclaimer": "本スコアは参考値です。投資判断は自己責任で行ってください。"
  }
}
```

**軸・Sub-score の構造:**

| 軸 | `sub[].id` | データソース |
|---|---|---|
| `disaster` (S1) | `flood`, `liquefaction`, `seismic`, `tsunami`, `landslide` | PostGIS + J-SHIS |
| `terrain` (S2) | `avs30` | J-SHIS |
| `livability` (S3) | `transit`, `education`, `medical` | PostGIS (transit は Phase 2) |
| `future` (S4) | `population`, `price_trend`, `far` | PostGIS (population は Phase 2) |
| `price` (S5) | `relative_value`, `volume` | PostGIS (z-score 比較) |

- `score`: 0-100 の数値。
- `available`: データ取得可否。`false` の場合は該当 sub を 100 (中立) として扱い、軸スコアに加味。
- `detail`: 軸ごとに異なる説明用フィールド (Zod は `z.record(z.string(), z.unknown())` なので **null を返さず `{}` を使うこと**)。
- `confidence`: その軸で `available=true` だった sub weight の合計。全 available なら 1.0。

**Grade 閾値** (`services/backend/src/domain/scoring/tls.rs` `Grade::from_score`):
`S≥85`, `A≥70`, `B≥55`, `C≥40`, `D≥25`, それ以下は `E`.

**Cross-analysis** (`compute_cross_analysis`):
- `value_discovery = S1 × (100 - v_rel) / 100` — 安全かつ割安
- `demand_signal   = S3 × S4 / 100` — 生活利便 × 将来性
- `ground_safety   = S1 × S2 / 100` — 災害 × 地盤

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

**Response 200** (Zod: `StatsResponse`):
```json
{
  "land_price": {
    "avg_per_sqm": 850000.0,
    "median_per_sqm": 720000.0,
    "min_per_sqm": 320000,
    "max_per_sqm": 3200000,
    "count": 45
  },
  "risk": {
    "flood_area_ratio": 0.15,
    "steep_slope_area_ratio": 0.02,
    "composite_risk": 0.18
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

> `land_price.{avg,median}_per_sqm` は `Option<f64>` (feature が無い bbox で `null`)。`risk.composite_risk` の古い別名 `avg_composite_risk` は削除済み。

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

### 3.6 GET /api/v1/land-prices

単年の地価公示データを bbox 内で取得する。`/api/area-data?layers=landprice` の代替として
独立したレート制限と年度指定に対応する専用エンドポイント。

**Query Parameters:**

| Param | Type | Required | Default | Description | Example |
|-------|------|----------|---------|-------------|---------|
| year | i32 | Yes | — | 公示年 (2020..=2024) | 2024 |
| bbox | string | Yes | — | `sw_lng,sw_lat,ne_lng,ne_lat` | `139.70,35.65,139.80,35.70` |
| zoom | u32 | No | 14 | 動的 feature limit に使用 | 15 |

**Response 200** (Zod: `LandPriceTimeSeriesResponse` = `layerResponse(LandPriceProperties)`):

```json
{
  "type": "FeatureCollection",
  "features": [
    {
      "type": "Feature",
      "geometry": { "type": "Polygon", "coordinates": [[...]] },
      "properties": {
        "id": 123,
        "price_per_sqm": 1200000,
        "address": "千代田区丸の内1-1",
        "land_use": "商業",
        "year": 2024
      }
    }
  ],
  "truncated": false,
  "count": 42,
  "limit": 500
}
```

> Point geometry は約 30m × 30m の polygon に変換されてから返される (MapLibre 3D extrusion 用)。
> `truncated=true` の場合、`limit` 件で切り詰められた。zoom in を促すこと。

実装: `services/backend/src/handler/land_price.rs`, `infra/pg_land_price_repository.rs::find_by_year_and_bbox`.

---

### 3.7 GET /api/v1/land-prices/all-years

Time machine スライダー用。指定 bbox 内の複数年地価を **1 リクエストで** 返す。
クライアントは MapLibre `setFilter(["==", ["get","year"], selectedYear])` で年度を
切り替えることで、年スクラブ時に再フェッチなしでアニメーションする。

**Query Parameters:**

| Param | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| bbox | string | Yes | — | `sw_lng,sw_lat,ne_lng,ne_lat` |
| from | i32 | No | 2019 | 開始年 (含む) |
| to | i32 | No | 2024 | 終了年 (含む) |
| zoom | u32 | No | 14 | 動的 limit 計算 (単年 limit × (to-from+1)) |

**バリデーション:** `from > to` は 400 エラー。

**Response 200**: `LandPriceTimeSeriesResponse` と同じ形 (§3.6 参照)。`features[].properties.year`
に各 feature の年度が入る。`limit` は年数倍に拡張されており、タイムアウトは 10 秒 (単年 5 秒)。

実装: `services/backend/src/handler/land_price_all_years.rs`,
`infra/pg_land_price_repository.rs::find_all_years_by_bbox`,
フロント: `services/frontend/src/features/land-prices/api/use-land-prices-all-years.ts`.

---

### 3.8 GET /api/area-stats

行政区コード (都道府県 2 桁 / 市区町村 5 桁) に対する集計統計。
`/api/stats` が任意 bbox を受け付けるのに対し、こちらは管理地域境界と連動する。

**Query Parameters:**

| Param | Type | Required | Description | Example |
|-------|------|----------|-------------|---------|
| code | string | Yes | 行政区コード (2 or 5 桁) | `13105` |

**Response 200** (Zod: `AreaStatsResponse`):

```json
{
  "code": "13105",
  "name": "文京区",
  "level": "municipality",
  "land_price": {
    "avg_per_sqm": 1020000.0,
    "median_per_sqm": 980000.0,
    "count": 128
  },
  "risk": {
    "flood_area_ratio": 0.08,
    "composite_risk": 0.12
  },
  "facilities": {
    "schools": 42,
    "medical": 187
  }
}
```

- `level`: `"prefecture"` or `"municipality"`
- `land_price.{avg,median}_per_sqm`: `Option<f64>` (データ無しで `null`)

実装: `services/backend/src/handler/area_stats.rs`,
`usecase/get_area_stats.rs`, `infra/pg_admin_area_stats_repository.rs`.

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

// TLS Response (5軸 + cross-analysis)
// 実体は services/backend/src/handler/response.rs を参照
#[derive(Serialize)]
pub struct TlsResponse {
    pub location: LocationDto,
    pub tls: TlsSummaryDto,   // score + grade + label
    pub axes: AxesDto,        // disaster / terrain / livability / future / price
    pub cross_analysis: CrossAnalysisDto,
    pub metadata: TlsMetadataDto,
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

---

## 7. レイヤースコープとクエリ契約

> 旧 `TERRASIGHT_SPEC_V1.md` から統合。クライアントがどの粒度でどのエンドポイントを
> 呼ぶかの契約を定義する。未実装部分は次フェーズの target state として扱う。

### 7.1 レイヤースコープ 3 類型

```ts
type LayerScope = "always_national" | "selected_prefecture" | "viewport";
```

| スコープ | 表示単位 | 選択状態依存 | zoom 依存 |
|---|---|---|---|
| `always_national` | 全国 | しない | style のみ変化 |
| `selected_prefecture` | 選択都道府県全域 | する | `zoom < 9` で national fallback (暫定閾値) |
| `viewport` | 実 viewport (`map.getBounds()`) | 場合による | 動的 limit |

### 7.2 現行レイヤー分類 (target)

| レイヤー | scope | 備考 |
|---|---|---|
| 行政界 | `always_national` | Base orientation layer |
| 鉄道路線 | `always_national` | static (FlatGeobuf) |
| 地形分類 | `always_national` | static |
| 断層線 | `always_national` | static |
| 火山 | `always_national` | static |
| 浸水履歴 | `selected_prefecture` | 都道府県全域 |
| 洪水浸水 | `selected_prefecture` | 都道府県全域 (現行 API) |
| 表層地質 | `selected_prefecture` | 都道府県全域 (static) |
| 土壌図 | `selected_prefecture` | 都道府県全域 (static) |
| 地価公示 | `viewport` | `/api/v1/land-prices?year=&bbox=` |
| 用途地域 / 学校 / 医療 / 液状化 / 土砂災害 | 未確定 | 後続フェーズで確定 |

### 7.3 bbox の source of truth

viewport 系クエリの bbox は **常に `map.getBounds()`** から取得した実 bbox とする。

**禁止事項:**
1. `latitude`/`longitude`/`zoom` からの近似 bbox 再計算
2. live `viewState` をそのまま query key に使うこと (debounce 必須)
3. 同一画面内で API query と WASM query が異なる bbox 契約を持つこと

### 7.4 クエリ発火タイミング

1. `onMove` 中は view state のみ更新
2. データクエリは `onMoveEnd + debounce` 後にのみ発火
3. debounce 時間は実装定数化 (現行 200ms)

目的: every-frame refetch 回避、query 数の予測可能化、`1 viewport action → 1 query cycle` の検証可能性。

### 7.5 Static layer 取得戦略

- static layer は **batched query** を唯一の標準入口とする (`useVisibleStaticLayers`)
- layer component 側の self-fetch は禁止
- 例外を許可する条件 (全て満たす場合のみ):
  1. batched query に乗せると UX または可用性が明確に悪化する
  2. データ責務が他 layer と共有されない
  3. duplicate fetch を起こさない設計とテストがある
  4. 例外理由が docs に明記されている

### 7.6 `selected_prefecture` レイヤーのクエリキー

`selected_prefecture` レイヤーの query キーは viewport bbox ではなく:

- `prefectureCode`
- `layerId`
- 必要なら `dataVersion`

`zoom < 9` の場合のみ national fallback 表示へ切り替える (暫定閾値、全国データ整備後に実測で再評価)。

### 7.7 Canonical field schema (target)

`AreaStatsResponse` を少なくとも `ward` を受け取れる shape に拡張する target:

```ts
const AreaStatsResponse = z.object({
  code: z.string(),
  level: z.enum(["prefecture", "municipality", "ward"]),
  prefName: z.string(),
  cityName: z.string().nullable(),
  wardName: z.string().nullable(),
  landPrice: z.object({
    avgPerSqm: z.number().nullable(),
    medianPerSqm: z.number().nullable(),
    count: z.number(),
  }),
  risk: z.object({
    floodAreaRatio: z.number(),
    compositeRisk: z.number(),
  }),
  facilities: z.object({ schools: z.number(), medical: z.number() }),
});
```

- camelCase / snake_case の最終選択は frontend Zod を先に確定してから backend DTO を合わせる
- **`name` 単一フィールドに集約しない** — UI で `wardName ?? cityName ?? prefName` の優先順で派生する

### 7.8 実装禁止事項

1. `viewState` から直接 query key を組み立てること (debounce 必須)
2. `selected_prefecture` レイヤーを viewport 断片で取得すること
3. `name` に依存して行政階層判定すること
4. 東京都道府県コード `"13"` を全国仕様の常設前提にすること
5. batched path と self-fetch path を無秩序に併存させること


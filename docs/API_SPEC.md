# API_SPEC.md — Terrasight REST API 仕様書

> Version: 3.0.0 | Updated: 2026-04-12
> Runtime: Rust Axum + Tokio + SQLx + PostGIS
> Base URL: `http://localhost:8000`
>
> **API contract source of truth**: Frontend Zod スキーマ (`services/frontend/src/lib/api/schemas/`)。
> Backend Response DTO と Zod スキーマのフィールド名は必ず一致させる。
> 差異は integration test で assert する。

---

## 1. 共通仕様

### 1.1 リクエスト/レスポンス形式

- Content-Type: `application/json`
- GeoJSON レスポンスは RFC 7946 準拠（座標は `[longitude, latitude]`）
- 文字エンコーディング: UTF-8
- 圧縮: gzip（tower-http CompressionLayer）

### 1.2 エラーレスポンス

```json
{
  "error": {
    "code": "INVALID_PARAMS",
    "message": "latitude must be between -90 and 90"
  }
}
```

| HTTP Status | コード | 説明 |
| --- | --- | --- |
| 400 | `INVALID_PARAMS` | パラメータ不正（型、範囲、都道府県/市区町村コード） |
| 400 | `BBOX_TOO_LARGE` | bbox 面積が 0.5 度四方を超過 |
| 404 | `NOT_FOUND` | リソースが見つからない |
| 408 | `TIMEOUT` | クエリタイムアウト |
| 503 | `DB_UNAVAILABLE` | PostGIS 接続不可 |

### 1.3 共通ヘッダー

```text
X-Request-Id: {uuid}
X-Response-Time: {ms}
Content-Type: application/json; charset=utf-8
Content-Encoding: gzip
```

### 1.4 共通パラメータ型

| 型 | 説明 | 例 |
| --- | --- | --- |
| `pref_code` | 都道府県コード（2桁、01–47） | `13` |
| `city_code` | JIS 市区町村コード（5桁） | `13101` |
| bbox | south, west, north, east の4パラメータ（各辺 ≤ 0.5°） | `south=35.65&west=139.70&north=35.70&east=139.80` |

### 1.5 実装参照

- Router: `services/backend/src/lib.rs`
- Handler: `services/backend/src/handler/`
- Value Objects: `services/backend/src/domain/value_object.rs`

---

## 2. エンドポイント一覧

| Method | Path | 概要 |
| --- | --- | --- |
| GET | `/api/health` | ヘルスチェック |
| GET | `/api/area-data` | 複数レイヤー GeoJSON 取得 |
| GET | `/api/area-stats` | 行政区コード別の集計統計 |
| GET | `/api/stats` | bbox 内統計集約 |
| GET | `/api/score` | 投資スコア算出（TLS） |
| GET | `/api/trend` | 地価推移データ |
| GET | `/api/v1/land-prices` | 地価公示（単年 + bbox） |
| GET | `/api/v1/land-prices/all-years` | 地価公示時系列 |
| GET | `/api/v1/opportunities` | 投資機会一覧 |
| GET | `/api/v1/transactions/summary` | 取引価格集計 |
| GET | `/api/v1/transactions` | 取引価格明細 |
| GET | `/api/v1/appraisals` | 鑑定評価 |
| GET | `/api/v1/municipalities` | 市区町村リスト |

---

## 3. エンドポイント詳細

### 3.1 GET /api/health

ヘルスチェック。DB 接続状態を返す。

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

---

### 3.2 GET /api/area-data

ビューポート（bbox）内の地理空間レイヤーデータを GeoJSON で取得。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| south | f64 | Yes | bbox 南端緯度 |
| west | f64 | Yes | bbox 西端経度 |
| north | f64 | Yes | bbox 北端緯度 |
| east | f64 | Yes | bbox 東端経度 |
| layers | string | Yes | カンマ区切りレイヤーID |
| zoom | u32 | Yes | ズームレベル（feature limit 制御） |
| pref_code | string | No | 都道府県フィルタ |

**有効レイヤーID:** `landprice`, `zoning`, `flood`, `steep_slope`, `schools`, `medical`

**Response 200:**

```json
{
  "landprice": { "type": "FeatureCollection", "features": [...] },
  "zoning": { "type": "FeatureCollection", "features": [...] }
}
```

---

### 3.3 GET /api/area-stats

行政区コード別の集計統計。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| area_code | string | Yes | 行政区コード |

---

### 3.4 GET /api/stats

bbox 内の統計集約（土地価格、リスク、施設、用途地域分布）。

**Query Parameters:** bbox (south/west/north/east) + optional pref_code

**Response 200:**

```json
{
  "land_price": {
    "avg_per_sqm": 850000,
    "median_per_sqm": 780000,
    "min_per_sqm": 320000,
    "max_per_sqm": 2100000,
    "count": 156
  },
  "risk": {
    "flood_area_ratio": 0.12,
    "steep_slope_area_ratio": 0.03,
    "composite_risk": 0.15
  },
  "facilities": { "schools": 23, "medical": 45 },
  "zoning_distribution": { "商業地域": 0.35, "住居地域": 0.45 }
}
```

---

### 3.5 GET /api/score

投資スコア算出（TLS: Total Location Score, 5 軸 + 4 プリセット）。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| lat | f64 | Yes | 緯度 |
| lng | f64 | Yes | 経度 |
| preset | string | No | `balance` / `investment` / `residential` / `disaster` |

---

### 3.6 GET /api/trend

指定座標周辺の地価推移データ。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| lat | f64 | Yes | 緯度 |
| lng | f64 | Yes | 経度 |
| years | i32 | No | 遡り年数（default: 10, max: 20） |

---

### 3.7 GET /api/v1/land-prices

指定年 + bbox 内の地価公示ポイント（GeoJSON）。

**Query Parameters:** bbox + `year` (u16) + `zoom` (u32) + optional `pref_code`

**Response 200:** GeoJSON FeatureCollection（Point geometry, `price_per_sqm` property）

---

### 3.8 GET /api/v1/land-prices/all-years

時系列地価公示（タイムマシン用）。

**Query Parameters:** bbox + `from_year` + `to_year` + `zoom` + optional `pref_code`

**Response 200:** GeoJSON FeatureCollection（`year` property 付き）

---

### 3.9 GET /api/v1/opportunities

投資機会一覧（フィルター + ページネーション）。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| bbox | string | Yes | `sw_lng,sw_lat,ne_lng,ne_lat` |
| preset | string | No | Weight preset |
| tls_min | u8 | No | TLS スコア下限 |
| risk_max | string | No | `low` / `mid` / `high` |
| price_min | i64 | No | 地価下限（円/m2） |
| price_max | i64 | No | 地価上限（円/m2） |
| zones | string | No | カンマ区切り用途地域コード |
| pref_code | string | No | 都道府県フィルタ |
| limit | u32 | No | 件数（default: 50, max: 50） |
| offset | u32 | No | オフセット |

---

### 3.10 GET /api/v1/transactions/summary

都道府県内の取引価格集計（マテリアライズドビュー `mv_transaction_summary`）。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| pref_code | string | Yes | 都道府県コード（2桁） |
| year_from | i32 | No | 開始年（2000–2100） |
| property_type | string | No | `condo` / `land_building` / `land` / `forest` / `agriculture` |

**Response 200:**

```json
[
  {
    "city_code": "13101",
    "transaction_year": 2024,
    "property_type": "condo",
    "tx_count": 156,
    "avg_total_price": 58000000,
    "median_total_price": 52000000,
    "avg_price_sqm": 1200000,
    "avg_area_sqm": 65,
    "avg_walk_min": 7
  }
]
```

**Zod Schema:** `TransactionSummarySchema` (`services/frontend/src/lib/api/schemas/transaction.ts`)

---

### 3.11 GET /api/v1/transactions

市区町村の取引価格明細。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| city_code | string | Yes | JIS 市区町村コード（5桁） |
| year_from | i32 | No | 開始年 |
| limit | u32 | No | 件数（default: 50, max: 200） |

**Response 200:**

```json
[
  {
    "city_code": "13101",
    "city_name": "千代田区",
    "district_name": "丸の内",
    "property_type": "land_building",
    "total_price": 120000000,
    "price_per_sqm": 1500000,
    "area_sqm": 80,
    "floor_plan": "3LDK",
    "building_year": 2015,
    "building_structure": "RC",
    "nearest_station": "東京",
    "station_walk_min": 5,
    "transaction_quarter": "2024Q3"
  }
]
```

**Zod Schema:** `TransactionDetailSchema`

---

### 3.12 GET /api/v1/appraisals

鑑定評価データ。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| pref_code | string | Yes | 都道府県コード（2桁） |
| city_code | string | No | JIS 市区町村コード（5桁） |

**Response 200:**

```json
[
  {
    "city_code": "13101",
    "city_name": "千代田区",
    "address": "丸の内一丁目1-1",
    "land_use_code": "商業",
    "price_per_sqm": 15000000,
    "appraisal_price": 750000000,
    "lot_area_sqm": 50.0,
    "zone_code": "商業地域",
    "building_coverage": 80,
    "floor_area_ratio": 800,
    "comparable_price": 14800000,
    "yield_price": 15200000,
    "cost_price": 14500000,
    "fudosan_id": "12345678901234567"
  }
]
```

**Zod Schema:** `AppraisalDetailSchema` (`services/frontend/src/lib/api/schemas/appraisal.ts`)

---

### 3.13 GET /api/v1/municipalities

都道府県内の市区町村リスト。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| pref_code | string | Yes | 都道府県コード（2桁） |

**Response 200:**

```json
[
  { "city_code": "13101", "city_name": "千代田区", "pref_code": "13" },
  { "city_code": "13102", "city_name": "中央区", "pref_code": "13" }
]
```

**Zod Schema:** `MunicipalitySchema` (`services/frontend/src/lib/api/schemas/municipality.ts`)

---

## 4. Planned (未実装)

### Phase 2（SaaS 化）

| Method | Path | 概要 |
| --- | --- | --- |
| POST | `/api/auth/register` | ユーザー登録 |
| POST | `/api/auth/login` | ログイン |
| POST | `/api/auth/refresh` | トークン更新 |
| GET | `/api/watchlist` | ウォッチリスト取得 |
| POST | `/api/watchlist` | ウォッチリスト追加 |
| DELETE | `/api/watchlist/:id` | ウォッチリスト削除 |

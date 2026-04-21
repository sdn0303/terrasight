# API_SPEC.md — Terrasight REST API 仕様書

> Version: 5.0.0 | Updated: 2026-04-21
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
- Domain Model: `services/backend/src/domain/model/`

---

## 2. エンドポイント一覧

| Method | Path | 概要 |
| --- | --- | --- |
| GET | `/api/v1/health` | ヘルスチェック |
| GET | `/api/v1/area-data` | 複数レイヤー GeoJSON 取得 |
| GET | `/api/v1/area-stats` | 行政区コード別の集計統計 |
| GET | `/api/v1/stats` | bbox 内統計集約 |
| GET | `/api/v1/score` | 投資スコア算出（TLS） |
| GET | `/api/v1/trend` | 地価推移データ |
| GET | `/api/v1/land-prices` | 地価公示（単年 + bbox） |
| GET | `/api/v1/land-prices/all-years` | 地価公示時系列 |
| GET | `/api/v1/land-prices/aggregation` | 地価ポリゴン集計 |
| GET | `/api/v1/opportunities` | 投資機会一覧 |
| GET | `/api/v1/transactions/summary` | 取引価格集計 |
| GET | `/api/v1/transactions` | 取引価格明細 |
| GET | `/api/v1/transactions/aggregation` | 取引事例ポリゴン集計 |
| GET | `/api/v1/appraisals` | 鑑定評価 |
| GET | `/api/v1/municipalities` | 市区町村リスト |
| GET | `/api/v1/population` | 市区町村別人口・世帯数 **(v3 NEW)** |
| GET | `/api/v1/vacancy` | 市区町村別空き家率 **(v3 NEW)** |
| GET | `/api/v1/vacancy/geo` | 空室率 + ポリゴン GeoJSON **(v3 NEW)** |
| GET | `/api/v1/population/geo` | 人口 + ポリゴン GeoJSON **(v3 NEW)** |

---

## 3. エンドポイント詳細

### 3.1 GET /api/v1/health

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

### 3.2 GET /api/v1/area-data

ビューポート（bbox）内の地理空間レイヤーデータを GeoJSON で取得。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| south | f64 | Yes | bbox 南端緯度 |
| west | f64 | Yes | bbox 西端経度 |
| north | f64 | Yes | bbox 北端緯度 |
| east | f64 | Yes | bbox 東端経度 |
| layers | string | Yes | カンマ区切りレイヤーID |
| zoom | u8 | Yes | ズームレベル（feature limit 制御） |
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

### 3.3 GET /api/v1/area-stats

行政区コード別の集計統計。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| area_code | string | Yes | 行政区コード |

---

### 3.4 GET /api/v1/stats

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

### 3.5 GET /api/v1/score

投資スコア算出（TLS: Total Location Score, 5 軸 + 4 プリセット）。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| lat | f64 | Yes | 緯度 |
| lng | f64 | Yes | 経度 |
| preset | string | No | `balance` / `investment` / `residential` / `disaster` (or `disaster_focus`) |

---

### 3.6 GET /api/v1/trend

指定座標周辺の地価推移データ。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| lat | f64 | Yes | 緯度 |
| lng | f64 | Yes | 経度 |
| years | i32 | No | 遡り年数（default: 10, max: 20） |

`direction` field values: `"up"` or `"down"`.

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
| property_type | string | No | MLIT 生文字列（例: `"宅地(土地)"`, `"中古マンション等"`） |

**Response 200:**

```json
[
  {
    "city_code": "13101",
    "transaction_year": 2024,
    "property_type": "中古マンション等",
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
| limit | u32 | No | 件数（default: 50, max: 200; 200 超過時はサーバー側で 200 にクランプ） |

**Response 200:**

```json
[
  {
    "city_code": "13101",
    "city_name": "千代田区",
    "district_name": "丸の内",
    "property_type": "宅地(土地と建物)",
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

### 3.14 GET /api/v1/land-prices/aggregation

市区町村ポリゴン別の地価集計。admin_boundaries と land_prices を空間結合し、
市区町村ごとの平均・中央値・最小・最大・件数・前年比を GeoJSON FeatureCollection で返却。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| south | f64 | Yes | bbox 南端緯度 |
| west | f64 | Yes | bbox 西端経度 |
| north | f64 | Yes | bbox 北端緯度 |
| east | f64 | Yes | bbox 東端経度 |
| pref_code | string | No | 都道府県フィルタ |

**Response 200:** GeoJSON FeatureCollection

```json
{
  "type": "FeatureCollection",
  "features": [{
    "type": "Feature",
    "geometry": { "type": "MultiPolygon", "coordinates": ["..."] },
    "properties": {
      "admin_code": "13101",
      "pref_name": "東京都",
      "city_name": "千代田区",
      "avg_price": 1250000.0,
      "median_price": 980000.0,
      "min_price": 320000.0,
      "max_price": 3180000.0,
      "count": 42,
      "prev_year_avg": 1130000.0,
      "change_pct": 10.6
    }
  }]
}
```

**Error codes:** 400 (`INVALID_PARAMS` — invalid bbox), 404 (`NOT_FOUND` — no data for area)

**Zod Schema:** `LandPriceAggregationResponse` (`services/frontend/src/lib/api/schemas/land-price-aggregation.ts`)

---

### 3.15 GET /api/v1/transactions/aggregation

市区町村ポリゴン別の取引事例集計。admin_boundaries と transaction_prices を
city_code で結合し、市区町村ごとの取引件数・平均単価・平均総額を GeoJSON FeatureCollection で返却。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| south | f64 | Yes | bbox 南端緯度 |
| west | f64 | Yes | bbox 西端経度 |
| north | f64 | Yes | bbox 北端緯度 |
| east | f64 | Yes | bbox 東端経度 |
| pref_code | string | No | 都道府県フィルタ |

**Response 200:** GeoJSON FeatureCollection

```json
{
  "type": "FeatureCollection",
  "features": [{
    "type": "Feature",
    "geometry": { "type": "MultiPolygon", "coordinates": ["..."] },
    "properties": {
      "admin_code": "13101",
      "city_name": "千代田区",
      "tx_count": 156,
      "avg_price_sqm": 850000.0,
      "avg_total_price": 42500000.0
    }
  }]
}
```

**Error codes:** 400 (`INVALID_PARAMS` — invalid bbox)

**Zod Schema:** `TransactionAggregationResponse` (`services/frontend/src/lib/api/schemas/transaction-aggregation.ts`)

---

## 4. v3 新規エンドポイント

### 4.1 GET /api/v1/population

市区町村別の人口・世帯数。e-Stat 国勢調査データに基づく。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| pref_code | string | Yes | 都道府県コード（2桁） |

**Response 200:**

```json
[
  {
    "city_code": "13104",
    "city_name": "新宿区",
    "population": 344880,
    "male": 172000,
    "female": 172880,
    "households": 206093,
    "census_year": 2020
  }
]
```

**Zod Schema:** `PopulationSchema` (`services/frontend/src/lib/api/schemas/population.ts`)

```typescript
export const PopulationSchema = z.object({
  city_code: z.string(),
  city_name: z.string(),
  population: z.number(),
  male: z.number(),
  female: z.number(),
  households: z.number(),
  census_year: z.number(),
});
```

---

### 4.2 GET /api/v1/vacancy

市区町村別の空き家率。住宅・土地統計調査データに基づく。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| pref_code | string | Yes | 都道府県コード（2桁） |

**Response 200:**

```json
[
  {
    "city_code": "13104",
    "city_name": "新宿区",
    "vacancy_count": 12500,
    "total_houses": 210000,
    "vacancy_rate_pct": 5.95,
    "survey_year": 2023
  }
]
```

**Zod Schema:** `VacancySchema` (`services/frontend/src/lib/api/schemas/vacancy.ts`)

```typescript
export const VacancySchema = z.object({
  city_code: z.string(),
  city_name: z.string(),
  vacancy_count: z.number(),
  total_houses: z.number(),
  vacancy_rate_pct: z.number(),
  survey_year: z.number(),
});
```

---

### 4.3 GET /api/v1/vacancy/geo

空室率データ + 行政区ポリゴン（GeoJSON）。admin_boundaries と vacancy_rates を空間結合。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| south | f64 | Yes | bbox 南端緯度 |
| west | f64 | Yes | bbox 西端経度 |
| north | f64 | Yes | bbox 北端緯度 |
| east | f64 | Yes | bbox 東端経度 |
| pref_code | string | No | 都道府県フィルタ |

**Response 200:** GeoJSON FeatureCollection

```json
{
  "type": "FeatureCollection",
  "features": [{
    "type": "Feature",
    "geometry": { "type": "MultiPolygon", "coordinates": ["..."] },
    "properties": {
      "admin_code": "13104",
      "city_name": "新宿区",
      "vacancy_rate_pct": 5.95,
      "vacancy_count": 12500,
      "total_houses": 210000,
      "survey_year": 2023
    }
  }]
}
```

---

### 4.4 GET /api/v1/population/geo

人口データ + 行政区ポリゴン（GeoJSON）。admin_boundaries と population_municipality を空間結合。

**Query Parameters:**

| Param | Type | Required | Description |
| --- | --- | --- | --- |
| south | f64 | Yes | bbox 南端緯度 |
| west | f64 | Yes | bbox 西端経度 |
| north | f64 | Yes | bbox 北端緯度 |
| east | f64 | Yes | bbox 東端経度 |
| pref_code | string | No | 都道府県フィルタ |

**Response 200:** GeoJSON FeatureCollection

```json
{
  "type": "FeatureCollection",
  "features": [{
    "type": "Feature",
    "geometry": { "type": "MultiPolygon", "coordinates": ["..."] },
    "properties": {
      "admin_code": "13104",
      "city_name": "新宿区",
      "population": 344880,
      "households": 206093,
      "census_year": 2020
    }
  }]
}
```

---

## 5. v3 既存エンドポイント拡張

### 5.1 area-data: stations レイヤー拡張

`GET /api/v1/area-data?layers=stations` レスポンスの Feature properties に `passenger_count` フィールドを追加。

```json
{
  "properties": {
    "station_name": "新宿",
    "line_name": "山手線",
    "operator_name": "JR東日本",
    "passenger_count": 775386
  }
}
```

### 5.2 area-data: medical レイヤー拡張

`GET /api/v1/area-data?layers=medical` レスポンスの Feature properties に `beds` フィールドを追加。

```json
{
  "properties": {
    "facility_name": "東京大学医学部附属病院",
    "facility_type": "総合病院",
    "beds": 1217
  }
}
```

**注**: `passenger_count` と `beds` は既にDB テーブルに存在するが、area-data handler のSELECT/DTO に含まれていない。handler + DTO の修正のみ。

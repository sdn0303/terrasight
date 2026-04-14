# WASM_SPEC.md — Terrasight WASM Spatial Engine 仕様書

> Version: 2.0.0 | Updated: 2026-04-13
> Crate: `terrasight-wasm` (`services/wasm/`)
> Runtime: Web Worker + wasm-bindgen
> Consumer: `services/frontend/src/lib/wasm/spatial-engine.ts`

---

## 1. Architecture

```text
Frontend (React)
  └─ SpatialEngineAdapter (spatial-engine.ts)
       └─ Web Worker (worker.ts)
            └─ SpatialEngine (Rust WASM via wasm-bindgen)
                 ├─ R-tree per layer (rstar)
                 ├─ FlatGeobuf parser (flatgeobuf + geozero)
                 ├─ GeoJSON parser (serde_json)
                 ├─ Area statistics (geo crate)
                 └─ TLS scoring (pure Rust)
```

- WASM バイナリは `/wasm/terrasight_wasm.js` + `.wasm` で配信
- Web Worker 内で初期化、メインスレッドとは postMessage 通信
- レイヤーデータは manifest.json 駆動で prefecture-scoped にロード

---

## 2. Public API (wasm-bindgen)

### 2.1 SpatialEngine

| Method | Signature | Description |
| --- | --- | --- |
| `new()` | `() -> SpatialEngine` | 空のエンジンを作成 |
| `load_layer` | `(layer_id: &str, fgb_bytes: &[u8]) -> Result<u32, JsValue>` | FlatGeobuf からレイヤーロード。feature 数を返す |
| `load_geojson_layer` | `(layer_id: &str, geojson: &str) -> Result<u32, JsValue>` | GeoJSON FeatureCollection からレイヤーロード |
| `query` | `(layer_id: &str, south, west, north, east: f64) -> Result<String, JsValue>` | 単一レイヤー bbox クエリ → GeoJSON FeatureCollection |
| `query_layers` | `(layer_ids: &str, south, west, north, east: f64) -> Result<String, JsValue>` | 複数レイヤー bbox クエリ → `{ layer_id: FC }` JSON |
| `compute_stats` | `(south, west, north, east: f64) -> Result<String, JsValue>` | bbox 内統計集約 → AreaStats JSON |
| `compute_tls` | `(south, west, north, east: f64, preset: &str) -> Result<String, JsValue>` | bbox 内 TLS スコア → TlsResult JSON |
| `feature_count` | `(layer_id: &str) -> u32` | ロード済み feature 数（未ロード = 0） |
| `loaded_layers` | `() -> String` | ロード済みレイヤー ID の JSON 配列 |

全メソッドで座標引数は `(south, west, north, east)` 順。内部で `BBox::new()` により検証。

---

## 3. Types

### 3.1 BBox

不変条件: `south < north`, `west < east`, 緯度 [-90, 90], 経度 [-180, 180]。
Private フィールド + getter。`constants.rs` の `LAT_RANGE` / `LNG_RANGE` で検証。

### 3.2 WasmError

```rust
enum WasmError {
    LayerNotFound(String),
    FgbOpen(String),
    FgbIteration(String),
    GeoJsonSerialise(String),
    Utf8(String),
    Json(serde_json::Error),  // #[from]
    GeoJsonParse(String),
    InvalidBBox(String),
}
```

JS 側には `JsValue::from_str(&e.to_string())` で文字列として伝播。

### 3.3 AreaStats (compute_stats 出力)

```json
{
  "land_price": {
    "avg_per_sqm": 850000.0,
    "median_per_sqm": 780000.0,
    "min_per_sqm": 320000.0,
    "max_per_sqm": 2100000.0,
    "count": 156
  },
  "risk": {
    "flood_area_ratio": 0.12,
    "steep_slope_area_ratio": 0.03,
    "composite_risk": 0.15
  },
  "facilities": {
    "schools": 23,
    "medical": 45,
    "stations_nearby": 8
  },
  "zoning_distribution": [
    { "zone": "商業地域", "ratio": 0.35 },
    { "zone": "住居地域", "ratio": 0.45 }
  ]
}
```

**Frontend Zod Schema:** `WasmStatsSchema` (`services/frontend/src/lib/api/schemas/wasm-stats.ts`)

### 3.4 TlsResult (compute_tls 出力)

```json
{
  "total_score": 0.72,
  "sub_scores": {
    "price_score": 0.85,
    "risk_score": 0.65,
    "facility_score": 0.70,
    "zoning_score": 0.55,
    "transport_score": 0.80
  }
}
```

### 3.5 WeightPreset

`WeightPreset` は `terrasight-domain::scoring::tls::WeightPreset` からインポートされ、バックエンドと共有。

| Preset | Price | Risk | Facility | Zoning | Transport |
| --- | --- | --- | --- | --- | --- |
| `balance` | 0.20 | 0.20 | 0.20 | 0.20 | 0.20 |
| `investment` | 0.35 | 0.15 | 0.10 | 0.25 | 0.15 |
| `residential` | 0.15 | 0.25 | 0.25 | 0.15 | 0.20 |
| `disaster` / `disaster_focus` | 0.10 | 0.40 | 0.15 | 0.15 | 0.20 |

不明な preset 文字列は `balance` にフォールバック（エラーにならない）。
共有 enum の variant 名は `DisasterFocus`。`FromStr` 実装は `"disaster"` と `"disaster_focus"` の両方を受け付ける。

### 3.6 NormalizationParams

| Param | Tokyo Default | Description |
| --- | --- | --- |
| `price_floor` | 300,000 | 地価下限（円/m2）、これ以下は最高スコア |
| `price_ceiling` | 3,000,000 | 地価上限（円/m2）、これ以上は最低スコア |
| `facility_cap` | 50 | 施設数上限、これ以上は満点 |
| `station_cap` | 10 | 駅数上限、これ以上は満点 |

将来的に都道府県別パラメータに拡張予定。

---

## 4. Layer Loading

### 4.1 Manifest-Driven

`/data/fgb/manifest.json` から prefecture 別レイヤーリストを取得。
**Zod Schema:** `ManifestSchema` (`services/frontend/src/lib/wasm/manifest-schema.ts`)

### 4.2 Prefecture Switching

`reloadForPrefecture(prefCode)`: Worker terminate → クリア → 新 prefCode で再 init。

### 4.3 GeoJSON Layer Injection

API データを `loadGeoJsonLayer(layerId, geojson)` で R-tree に投入し、stats/TLS 計算に参加。

---

## 5. Worker Message Protocol

| Direction | Type | Payload |
| --- | --- | --- |
| Main → Worker | `init` | `{ layers: [{id, url}] }` |
| Main → Worker | `query` | `{ id, layerIds, bbox }` |
| Main → Worker | `compute-stats` | `{ id, bbox }` |
| Main → Worker | `compute-tls` | `{ id, bbox, preset }` |
| Main → Worker | `load-geojson` | `{ id, layerId, geojson }` |
| Worker → Main | `init-done` | `{ layerCounts }` |
| Worker → Main | `query-result` | `{ id, data }` |
| Worker → Main | `stats-result` / `tls-result` / `load-geojson-result` | `{ id, data/result/count }` |
| Worker → Main | `*-error` | `{ id, error }` |

---

## 6. Performance Targets

| Operation | Target (p95) |
| --- | --- |
| `init` (全レイヤーロード) | < 1s |
| `query` (5 layers) | < 16ms |
| `compute_stats` | < 16ms |
| `compute_tls` | < 5ms |
| `load_geojson_layer` | < 100ms / layer |

---

## 7. Constants Module (`src/constants.rs`)

全マジックナンバーは `pub(crate) const` で集約:

- **Layer IDs**: `LAYER_LANDPRICE`, `LAYER_FLOOD_HISTORY`, `LAYER_FLOOD`, `LAYER_STEEP_SLOPE`, `LAYER_STEEP_SLOPE_ALT`, `LAYER_SCHOOLS`, `LAYER_MEDICAL`, `LAYER_RAILWAY`, `LAYER_STATION`, `LAYER_ZONING`
- **JSON keys**: `PROP_PRICE_PER_SQM`, `PROP_ZONE_TYPE`
- **GeoJSON**: `GEOJSON_KEY_TYPE`, `GEOJSON_KEY_GEOMETRY`, `GEOJSON_KEY_PROPERTIES`, `GEOJSON_KEY_COORDINATES`, `GEOJSON_KEY_GEOMETRIES`, `GEOJSON_TYPE_POINT`, `GEOJSON_TYPE_LINE_STRING`, `GEOJSON_TYPE_POLYGON`, `GEOJSON_TYPE_MULTI_POLYGON`, `FC_HEADER`, `FC_FOOTER`
- **WGS84**: `LAT_MIN`, `LAT_MAX`, `LNG_MIN`, `LNG_MAX`, `LAT_RANGE`, `LNG_RANGE`
- **Risk**: `RISK_WEIGHT_FLOOD`, `RISK_WEIGHT_STEEP`（`terrasight_domain::constants` から re-export）
- **Zoning**: `COMMERCIAL_ZONE_KEYWORD`
- **Coordinate**: `MIN_COORD_PAIR_LEN`

---

## 8. Visibility Rules

| Scope | Visibility | Examples |
| --- | --- | --- |
| JS API | `pub` + `#[wasm_bindgen]` | `SpatialEngine`, `BBox`, `WasmError` |
| Crate internal | `pub(crate)` | `LayerIndex`, `ParsedFeature`, `AreaStats`, constants |
| Module internal | private | inner methods, helpers |

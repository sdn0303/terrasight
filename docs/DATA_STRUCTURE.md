# Data Structure

Terrasight のデータ構造。3層で構成: PostGIS (API用), FlatGeobuf (WASM用), Pipeline (変換)。

## Overview

```
data/raw/                          ← 生データ (KSJ ZIP + REINFOLIB CSV)
  ├── N03-*.zip, A29-*, L01-*, ... ← 国土数値情報 (GeoJSON/Shapefile)
  └── 不動産情報ライブラリ/         ← REINFOLIB (CSV, CP932)
      ├── 不動産価格（取引価格・成約価格）情報/
      └── 鑑定評価書情報地価公示/

    ↓ pipeline.sh (convert → build_fgb → import → import_reinfolib)

data/geojson/{pref_code}/         ← 変換済み GeoJSON (中間ファイル)
data/fgb/{pref_code}/             ← FlatGeobuf (WASM 空間エンジン用)
data/fgb/manifest.json            ← レイヤーカタログ
PostgreSQL + PostGIS              ← API レスポンス用
```

## Layer 1: PostGIS Tables

全テーブル共通: `pref_code text NOT NULL` で都道府県フィルタ、`created_at timestamptz NOT NULL DEFAULT now()`。

### Spatial Tables (geometry あり)

#### admin_boundaries — 行政区画

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| level | text | `'prefecture'` or `'municipality'` |
| pref_code | text | 2桁ゼロ埋め (`'13'`) |
| pref_name | text | `'東京都'` |
| city_code | text | 5桁 JIS (`'13101'`), prefecture の場合 NULL |
| city_name | text | `'千代田区'` |
| admin_code | text | N03_007 から変換 |
| geom | MultiPolygon, 4326 | GiST index |
| area_sqm | double precision | |

Source: KSJ N03 → GeoJSON → import
Rows (Tokyo): 6,904

#### land_prices — 地価公示

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| address | text | `'東京都　千代田区三番町６番２５'` |
| price_per_sqm | integer | CHECK >= 0 |
| land_use | text | 土地利用区分 |
| zone_type | text | 用途地域 |
| survey_year | smallint | CHECK 2000-2100 |
| geom | Point, 4326 | GiST index |

Source: KSJ L01 → GeoJSON (v3.0/v3.1 自動判定, canonical field 正規化) → import
Rows (Tokyo): 2,560
UNIQUE: `(address, survey_year)`

#### zoning — 用途地域

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| zone_code | text | A29_004 |
| zone_type | text | A29_005 |
| floor_area_ratio | double precision | |
| building_coverage | double precision | |
| geom | MultiPolygon, 4326 | GiST index |

Source: KSJ A29 (per-pref ZIP) → import
Rows (Tokyo): 11,164

#### schools — 学校

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| school_name | text | |
| school_type | text | P29_001 |
| address | text | |
| geom | Point, 4326 | GiST index |

Source: KSJ P29 → import
Rows (Tokyo): 4,449

#### medical_facilities — 医療施設

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| facility_name | text | P04_002 |
| facility_type | text | P04_004 (診療科目) |
| beds | integer | P04_008 |
| address | text | P04_003 |
| geom | Point, 4326 | GiST index |

Source: KSJ P04 → GeoJSON (UTF-8優先) → import
Rows (Tokyo): 25,424

#### stations — 駅

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| station_name | text | S12_001 |
| station_code | text | S12_001c (6桁駅コード) |
| line_name | text | S12_003 |
| operator_name | text | S12_002 |
| passenger_count | integer | S12_004 |
| geom | Point, 4326 | LineString → centroid 変換 |

Source: KSJ S12 → UTF-8 GeoJSON → 空間 bbox フィルタ → centroid → import
Rows (Tokyo): 1,133

#### railways — 鉄道路線

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| line_name | text | N02_003 |
| operator_name | text | N02_004 |
| railway_type | text | N02_002 |
| geom | MultiLineString, 4326 | LineString → Multi 変換 |

Source: KSJ N02 → UTF-8 GeoJSON (`RailroadSection` hint) → 空間 bbox フィルタ → import
Rows (Tokyo): 2,423

#### flood_risk — 洪水浸水想定

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| depth_rank | text | |
| river_name | text | |
| geom | MultiPolygon, 4326 | |

Source: KSJ A31b (未実装, P2)

#### steep_slope — 急傾斜地

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| area_name | text | |
| geom | MultiPolygon, 4326 | |

Source: KSJ A47 (未テスト, P2)

#### seismic_hazard — 地震ハザード

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| hazard_level | text | |
| probability | double precision | |
| geom | MultiLineString, 4326 | |

Source: J-SHIS

#### liquefaction — 液状化リスク

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| pl_rank | text | |
| geom | Point, 4326 | |

Source: メッシュデータ (P2)

#### population_municipality — 市区町村別人口 **(v3 NEW)**

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | 2桁ゼロ埋め |
| city_code | text | 5桁 JIS |
| city_name | text | |
| population | integer | 総人口 |
| male | integer | 男性人口 |
| female | integer | 女性人口 |
| households | integer | 世帯数 |
| census_year | smallint | 国勢調査年 |

Source: e-Stat 国勢調査 → `import_estat.py`
UNIQUE: `(pref_code, city_code, census_year)`

#### vacancy_rates — 空き家率 **(v3 NEW)**

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | 2桁ゼロ埋め |
| city_code | text | 5桁 JIS |
| city_name | text | |
| vacancy_count | integer | 空き家数 |
| total_houses | integer | 総住宅数 |
| vacancy_rate_pct | real | 空き家率 (%) |
| survey_year | smallint | 調査年 |

Source: e-Stat 住宅・土地統計調査 → `import_estat.py`
UNIQUE: `(pref_code, city_code, survey_year)`

### Non-Spatial Tables (geometry なし)

#### transaction_prices — 不動産取引価格

**Partitioned by LIST (pref_code)**, 47 子テーブル (`transaction_prices_01` 〜 `transaction_prices_47`)。

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK (composite: id + pref_code) |
| pref_code | text | パーティションキー |
| city_code | text | 5桁 JIS |
| city_name | text | |
| district_name | text | 地区名 |
| property_type | text | CHECK: `condo`, `land_building`, `land`, `forest`, `agriculture` |
| price_category | text | CHECK: `transaction` (取引価格), `contract` (成約価格) |
| total_price | bigint | CHECK > 0 |
| price_per_sqm | integer | 85% が NULL (土地のみ取引) |
| area_sqm | integer | 5㎡ 刻みにビニング済み |
| floor_plan | text | `'2LDK'` 等 |
| building_year | smallint | 西暦変換済み, `戦前` → 1945 |
| building_structure | text | `'ＲＣ'`, `'木造'`, `'ＳＲＣ'` 等 |
| current_use | text | `'住宅'`, `'共同住宅'` 等 |
| city_planning_zone | text | `'商業'`, `'１低住専'` 等 |
| building_coverage | smallint | % |
| floor_area_ratio | smallint | % |
| nearest_station | text | 最寄駅名 |
| station_walk_min | smallint | 特殊値変換済み: `30分〜60分` → 45 |
| front_road_width | real | m |
| land_shape | text | |
| transaction_quarter | text | `'2024Q3'` |
| transaction_year | smallint | CHECK 2000-2100 |
| transaction_q | smallint | CHECK 1-4 |

Source: REINFOLIB CSV (CP932, ヘッダあり) → `reinfolib_csv.py` → `import_db.py --reinfolib`
Rows (Tokyo): 729,098 | Total: ~6,340,000
Period: 2005 Q3 – 2025 Q3

Indexes:
- `idx_tx_city_year (city_code, transaction_year)`
- `idx_tx_pref_type_year (pref_code, property_type, transaction_year)`
- `idx_tx_quarter (transaction_year, transaction_q)`

#### land_appraisals — 鑑定評価書

| Column | Type | Note |
|--------|------|------|
| id | bigint (identity) | PK |
| pref_code | text | |
| city_code | text | pref_code + 3桁市区町村コード |
| city_name | text | |
| land_use_code | text | `'00'`=住宅地, `'05'`=商業地 |
| sequence_no | smallint | 市区町村×用途内の連番 |
| appraiser_no | smallint | 評価員番号 |
| survey_year | smallint | CHECK 2000-2100 |
| appraisal_price | bigint | 鑑定評価額 (円), CHECK > 0 |
| price_per_sqm | integer | 1㎡単価, CHECK > 0 |
| address | text | 所在地番 |
| display_address | text | 住居表示 |
| lot_area_sqm | real | 地積 (㎡) |
| current_use_code | text | 現況コード |
| zone_code | text | 用途地域コード |
| building_coverage | smallint | 指定建ぺい率 % |
| floor_area_ratio | smallint | 指定容積率 % |
| nearest_station | text | |
| station_distance_m | integer | 駅距離 (m) |
| front_road_width | real | 前面道路幅員 (m) |
| fudosan_id | text | 17桁不動産ID (将来の座標紐付け用) |
| comparable_price | integer | 比準価格 (円/㎡) |
| yield_price | integer | 収益価格 (円/㎡) |
| cost_price | integer | 積算価格 (円/㎡) |

Source: REINFOLIB CSV (CP932, ヘッダなし, 1408列中35列抽出) → `reinfolib_csv.py` → `import_db.py --reinfolib`
Rows (Tokyo): 2,552 (5,104 読み込み → UNIQUE 制約で重複排除)
UNIQUE: `(pref_code, city_code, land_use_code, sequence_no, appraiser_no, survey_year)`

Indexes:
- `idx_appr_pref_year (pref_code, survey_year)`
- `idx_appr_city (city_code)`
- `idx_appr_fudosan (fudosan_id) WHERE fudosan_id IS NOT NULL` — partial index

### Materialized Views

#### mv_transaction_summary

市区町村×年×物件種別の集計。L2 チョロプレス表示用。

| Column | Type | Note |
|--------|------|------|
| pref_code | text | |
| city_code | text | |
| transaction_year | smallint | |
| property_type | text | |
| tx_count | integer | 取引件数 |
| avg_total_price | bigint | 平均取引価格 (round済み) |
| median_total_price | bigint | 中央値 (percentile_cont) |
| avg_price_sqm | integer | 平均㎡単価 |
| avg_area_sqm | integer | 平均面積 |
| avg_walk_min | smallint | 平均駅徒歩 |

UNIQUE: `(pref_code, city_code, transaction_year, property_type)`
Rows (Tokyo): 3,521
Refresh: `REFRESH MATERIALIZED VIEW CONCURRENTLY` (autocommit required)

#### mv_appraisal_summary

市区町村×用途の鑑定評価集計。

| Column | Type | Note |
|--------|------|------|
| pref_code | text | |
| city_code | text | |
| land_use_code | text | |
| survey_year | smallint | |
| parcel_count | integer | 地点数 |
| avg_price_sqm | integer | 平均㎡単価 |
| avg_lot_area | real | 平均地積 |
| avg_comparable | integer | 平均比準価格 |
| avg_yield | integer | 平均収益価格 |
| avg_cost | integer | 平均積算価格 |

UNIQUE: `(pref_code, city_code, land_use_code, survey_year)`
Rows (Tokyo): 134

## Layer 2: FlatGeobuf (WASM 空間エンジン)

`data/fgb/{pref_code}/` に配置。`manifest.json` でカタログ管理。WASM R-tree にロードされ、クライアントサイド空間クエリ + TLS スコア計算に使用。

### Per-Prefecture Layers

| Layer ID | File | Source | Note |
|----------|------|--------|------|
| admin-boundary | `{pref}/admin-boundary.fgb` | Pipeline (N03) | L1/L2 境界ポリゴン |
| railway | `{pref}/railway.fgb` | Pipeline (N02) | 鉄道路線 |
| flood-history | `{pref}/flood-history.fgb` | Manual static | 浸水履歴 |
| geology | `{pref}/geology.fgb` | Manual static | 地質 |
| landform | `{pref}/landform.fgb` | Manual static | 地形分類 |
| soil | `{pref}/soil.fgb` | Manual static | 土壌 |
| did | `{pref}/did.fgb` | Manual static | 人口集中地区 |
| liquefaction | `{pref}/liquefaction.fgb` | Manual static | 液状化リスク |

### National Layers

| Layer ID | File | Source |
|----------|------|--------|
| fault | `national/fault.fgb` | Manual static |
| volcano | `national/volcano.fgb` | Manual static |
| seismic | `national/seismic.fgb` | Manual static |

### Manifest Format

```json
{
  "version": "2.0.0",
  "prefectures": {
    "13": {
      "layers": [
        { "id": "admin-boundary", "path": "13/admin-boundary.fgb", "features": 6904, "size_bytes": 8768208 }
      ]
    },
    "national": { "layers": [...] }
  }
}
```

## Layer 3: Pipeline Data Flow

### Data Sources

| Source | Code | Format | Encoding | Geometry | Adapter |
|--------|------|--------|----------|----------|---------|
| 国土数値情報 (KSJ) | N03, A29, L01, P29, P04, S12, N02 | Shapefile + GeoJSON in ZIP | UTF-8 / CP932 | Yes | `NationalArchiveAdapter` / `PerPrefArchiveAdapter` |
| 不動産情報ライブラリ | — | CSV in ZIP | CP932 | No | `reinfolib_csv.py` (直接呼び出し) |

### KSJ Format Handling

| Issue | Solution |
|-------|----------|
| v3.1 Shapefile 属性破損 (144+ cols) | GeoJSON-first reader (`zip_utils.py`) |
| L01 v3.0/v3.1 フィールド番号変更 | バージョン自動判定 + canonical field 正規化 |
| S12/N02 都道府県コードなし | 都道府県 bbox 空間フィルタ |
| S12 LineString → stations Point | Centroid 変換 (import_db.py) |
| N02 LineString → railways MultiLineString | Multi ラッピング (import_db.py) |
| Shift-JIS/UTF-8 デュアルディレクトリ | UTF-8 優先選択 |
| N02 複数 GeoJSON (Station + Section) | `geojson_hint: "RailroadSection"` |
| Shapefile CP932 文字化け | `latin-1 → cp932` 再エンコード正規化 |

### REINFOLIB CSV Handling

| Issue | Solution |
|-------|----------|
| 数値列に文字列 (`30分〜60分`, `2000㎡以上`) | 専用パーサで変換 |
| `戦前` (建築年) | → 1945 |
| 和暦 (`昭和63年`) | → 西暦 (1988) ※現データには存在しないが防御的対応 |
| 1408列 CSV (鑑定評価書) | 35列のみ抽出 |
| 奈良県 ZIP 欠落 | graceful skip |

### Pipeline Steps

```
pipeline.sh {pref_code} {priority}
  ├── Step 0:  Schema migration (idempotent)
  ├── Step 0b: REINFOLIB migration (idempotent)
  ├── Step 1:  convert.py   — RAW → GeoJSON (7 datasets)
  ├── Step 2:  build_fgb.py — GeoJSON → FlatGeobuf + manifest
  ├── Step 3:  import_db.py — GeoJSON → PostGIS (7 tables)
  ├── Step 3b: import_db.py --reinfolib — CSV → PostGIS (2 tables + 2 matviews)
  └── Step 4:  validate.py  — GeoJSON + FGB 件数チェック
```

### Dataset Catalog

`data/catalog/dataset_catalog.json` で全データセットを管理。

| ID | KSJ Code | Adapter | DB Table | Priority |
|----|----------|---------|----------|----------|
| admin-boundary | N03 | NationalArchiveAdapter | admin_boundaries | P0 |
| zoning | A29 | PerPrefArchiveAdapter | zoning | P0 |
| land-price | L01 | NationalArchiveAdapter | land_prices | P0 |
| schools | P29 | PerPrefArchiveAdapter | schools | P0 |
| medical | P04 | PerPrefArchiveAdapter | medical_facilities | P0 |
| stations | S12 | NationalArchiveAdapter | stations | P0 |
| railway | N02 | NationalArchiveAdapter | railways | P0 |
| transaction-prices | — | null (直接) | transaction_prices | P0 |
| land-appraisal | — | null (直接) | land_appraisals | P0 |
| flood | A31b | PerPrefArchiveAdapter | flood_risk | P2 |
| steep-slope | A47 | PerPrefArchiveAdapter | steep_slope | P2 |

## Row Counts (pref_code = '13', Tokyo)

| Table | Rows | Source |
|-------|------|--------|
| admin_boundaries | 6,904 | KSJ N03 |
| zoning | 11,164 | KSJ A29 |
| land_prices | 2,560 | KSJ L01 (v3.1) |
| schools | 4,449 | KSJ P29 |
| medical_facilities | 25,424 | KSJ P04 |
| stations | 1,133 | KSJ S12 (空間フィルタ) |
| railways | 2,423 | KSJ N02 (空間フィルタ) |
| transaction_prices | 729,098 | REINFOLIB CSV |
| land_appraisals | 2,552 | REINFOLIB CSV |
| mv_transaction_summary | 3,521 | Materialized view |
| mv_appraisal_summary | 134 | Materialized view |

## Migrations

| File | Content |
|------|---------|
| `00000000000001_schema.sql` | KSJ 11 tables + indexes |
| `00000000000002_reinfolib.sql` | REINFOLIB 2 tables (1 partitioned) + 2 materialized views |

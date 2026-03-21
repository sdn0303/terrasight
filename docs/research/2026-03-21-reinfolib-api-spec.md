# 不動産情報ライブラリ API 依存仕様書

> **調査日:** 2026-03-21
> **ソース:** https://www.reinfolib.mlit.go.jp/help/apiManual/
> **ステータス:** APIキー申請中（2026-03-18 申請）

---

## 1. 共通仕様

### 1.1 認証

| 項目 | 値 |
|------|-----|
| ヘッダー名 | `Ocp-Apim-Subscription-Key` |
| 値 | 発行されたAPIキー文字列 |
| 申請先 | https://www.reinfolib.mlit.go.jp/ |

### 1.2 ベースURL

```
https://www.reinfolib.mlit.go.jp/ex-api/external/{ENDPOINT_ID}
```

### 1.3 レート制限

- 明示的な数値制限は非公開
- 「同一APIキーで基準期間内に多数のリクエストがあった場合にはアクセス制限」
- 推奨: リクエスト間隔を空ける、連続実行を避ける

### 1.4 レスポンス形式

| 形式 | `response_format` 値 | 用途 |
|------|---------------------|------|
| GeoJSON | `geojson` | データ変換・分析向け |
| PBF | `pbf` | マップ描画最適化向け |

### 1.5 タイル座標系

- **方式**: XYZ スリッピーマップタイル
- **参照**: https://maps.gsi.go.jp/development/tileCoordCheck.html
- **ズームレベル**: エンドポイントにより 11-15 または 13-15

### 1.6 エラーハンドリング

| 状況 | 動作 |
|------|------|
| タイル内データなし | HTTP 200 + 空の features 配列 |
| 非タイルAPIデータなし | HTTP 404 |
| レート制限超過 | HTTP 429（推定） |
| 認証エラー | HTTP 401（推定） |

### 1.7 注意事項

- ブラウザから直接APIリクエストを送信するとCORSエラーが発生する
- バックエンドサーバー経由でリクエストすること
- gzip エンコーディングのレスポンスあり（クライアント側で解凍が必要な場合あり）

---

## 2. エンドポイント詳細

### 2.1 XPT001 — 不動産取引価格ポイント

**用途**: 取引価格データを地図タイル上のポイントとして取得

| 項目 | 値 |
|------|-----|
| URL | `https://www.reinfolib.mlit.go.jp/ex-api/external/XPT001` |
| メソッド | GET |
| ズームレベル | 11-15 |
| データ期間 | 取引価格: 2005年Q3〜、成約価格: 2021年Q1〜 |

#### パラメータ

| パラメータ | 型 | 必須 | 有効値 | 説明 |
|-----------|-----|------|--------|------|
| `response_format` | string | Yes | `geojson`, `pbf` | レスポンス形式 |
| `z` | integer | Yes | 11-15 | ズームレベル |
| `x` | integer | Yes | — | タイルX座標 |
| `y` | integer | Yes | — | タイルY座標 |
| `from` | string | Yes | `YYYYN` (5桁) | 取引期間開始（YYYY=年, N=四半期1-4） |
| `to` | string | Yes | `YYYYN` (5桁) | 取引期間終了 |
| `priceClassification` | string | No | `01`, `02` | 01=取引のみ, 02=成約のみ, 省略=両方 |
| `landTypeCode` | string | No | `01,02,07,10,11` | 土地種別（カンマ区切り可） |

#### 土地種別コード

| コード | 種別 |
|--------|------|
| 01 | 土地 |
| 02 | 土地と建物 |
| 07 | 中古マンション |
| 10 | 農地 |
| 11 | 林地 |

#### レスポンスプロパティ（GeoJSON）

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `price_information_category_name_ja` | string | 価格情報種別 |
| `district_code` | string | 地区コード |
| `city_code` | string | 市区町村コード |
| `prefecture_name_ja` | string | 都道府県名 |
| `city_name_ja` | string | 市区町村名 |
| `district_name_ja` | string | 地区名 |
| `u_transaction_price_total_ja` | string | 取引価格（総額） |
| `u_unit_price_per_tsubo_ja` | string | 坪単価 |
| `floor_plan_name_ja` | string | 間取り |
| `u_area_ja` | string | 面積 |
| `u_transaction_price_unit_price_square_meter_ja` | string | ㎡単価 |
| `land_shape_name_ja` | string | 土地形状 |
| `u_land_frontage_ja` | string | 間口 |
| `u_building_total_floor_area_ja` | string | 延床面積 |
| `u_construction_year_ja` | string | 建築年 |
| `building_structure_name_ja` | string | 建物構造 |
| `land_use_name_ja` | string | 用途地域名 |
| `future_use_purpose_name_ja` | string | 今後の利用目的 |
| `front_road_azimuth_name_ja` | string | 前面道路方位 |
| `front_road_type_name_ja` | string | 前面道路種類 |
| `u_front_road_width_ja` | string | 前面道路幅員 |
| `u_building_coverage_ratio_ja` | string | 建蔽率 |
| `u_floor_area_ratio_ja` | string | 容積率 |
| `point_in_time_name_ja` | string | 取引時点 |
| `land_type_name_ja` | string | 土地種別名 |

#### リクエスト例

```bash
curl -H "Ocp-Apim-Subscription-Key:{KEY}" \
  "https://www.reinfolib.mlit.go.jp/ex-api/external/XPT001?response_format=geojson&z=14&x=14624&y=6016&from=20252&to=20252"
```

---

### 2.2 XPT002 — 地価公示ポイント

**用途**: 地価公示・地価調査データをポイントとして取得

| 項目 | 値 |
|------|-----|
| URL | `https://www.reinfolib.mlit.go.jp/ex-api/external/XPT002` |
| メソッド | GET |
| ズームレベル | 13-15 |
| データ期間 | 地価公示: 1995年〜、地価調査: 1997年〜 |

#### パラメータ

| パラメータ | 型 | 必須 | 有効値 | 説明 |
|-----------|-----|------|--------|------|
| `response_format` | string | Yes | `geojson`, `pbf` | レスポンス形式 |
| `z` | integer | Yes | 13-15 | ズームレベル |
| `x` | integer | Yes | — | タイルX座標 |
| `y` | integer | Yes | — | タイルY座標 |
| `year` | string | Yes | `NNNN` (4桁) | 対象年（1995-2024+） |
| `priceClassification` | digit | No | `0`, `1` | 0=地価公示のみ, 1=地価調査のみ, 省略=両方 |
| `useCategoryCode` | string | No | `00,03,05,07,09,10,13,20` | 用途区分コード（カンマ区切り可） |

#### 用途区分コード

| コード | 区分 |
|--------|------|
| 00 | 住宅地 |
| 03 | 宅地見込地 |
| 05 | 商業地 |
| 07 | 準工業地 |
| 09 | 工業地 |
| 10 | 市街化調整区域内宅地 |
| 13 | 市街化調整区域内林地 |
| 20 | 林地 |

#### レスポンスプロパティ（主要フィールド、50+項目）

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `point_id` | string | 地点ID |
| `target_year_name_ja` | string | 対象年度 |
| `land_price_type` | string | 地価種別 |
| `prefecture_name_ja` | string | 都道府県名 |
| `city_name_ja` | string | 市区町村名 |
| `u_current_year_price_ja` | string | 当年価格 |
| `u_previous_year_price_ja` | string | 前年価格 |
| `u_change_rate_ja` | string | 変動率 |
| `use_area_ja` | string | 用途地域 |
| `u_floor_area_ratio_ja` | string | 容積率 |
| `u_building_coverage_ratio_ja` | string | 建蔽率 |
| `nearest_station_name_ja` | string | 最寄り駅名 |

---

### 2.3 XIT001 — 不動産取引価格情報（非タイル）

**用途**: 地域指定による取引価格データの一括取得（地図タイルではない）

| 項目 | 値 |
|------|-----|
| URL | `https://www.reinfolib.mlit.go.jp/ex-api/external/XIT001` |
| メソッド | GET |
| データ期間 | 取引: 2005年〜、成約: 2021年〜 |
| 圧縮 | gzip（クライアント側解凍必要） |

#### パラメータ

| パラメータ | 型 | 必須 | 有効値 | 説明 |
|-----------|-----|------|--------|------|
| `year` | string | Yes | `YYYY` | 対象年 |
| `quarter` | integer | Yes | 1-4 | 対象四半期 |
| `area` | string | Conditional* | 2桁都道府県コード | 都道府県（カンマ区切り可） |
| `city` | string | Conditional* | 5桁市区町村コード | 市区町村 |
| `station` | string | Conditional* | 6桁駅コード | 鉄道駅 |
| `priceClassification` | string | No | `01`, `02` | 01=取引, 02=成約, 省略=両方 |
| `language` | string | No | `ja`, `en` | レスポンス言語（デフォルト: ja） |

*`area`, `city`, `station` のいずれか1つ以上を指定必須

#### レスポンスフィールド（全 string 型）

`Type`, `Region`, `MunicipalityCode`, `Prefecture`, `Municipality`, `DistrictName`, `TradePrice`, `PricePerUnit`, `FloorPlan`, `Area`, `UnitPrice`, `LandShape`, `Frontage`, `TotalFloorArea`, `BuildingYear`, `Structure`, `Use`, `Purpose`, `Direction`, `Classification`, `Breadth`, `CityPlanning`, `CoverageRatio`, `FloorAreaRatio`, `Period`, `Renovation`, `Remarks`, `PriceCategory`, `DistrictCode`

---

### 2.4 XKT002 — 用途地域

**用途**: 都市計画の用途地域ポリゴンデータを取得

| 項目 | 値 |
|------|-----|
| URL | `https://www.reinfolib.mlit.go.jp/ex-api/external/XKT002` |
| メソッド | GET |
| ズームレベル | 11-15 |

#### パラメータ

| パラメータ | 型 | 必須 | 有効値 | 説明 |
|-----------|-----|------|--------|------|
| `response_format` | string | Yes | `geojson`, `pbf` | レスポンス形式 |
| `z` | integer | Yes | 11-15 | ズームレベル |
| `x` | integer | Yes | — | タイルX座標 |
| `y` | integer | Yes | — | タイルY座標 |

#### レスポンスプロパティ

| フィールド | 型 | 説明 | 例 |
|-----------|-----|------|-----|
| `youto_id` | integer | 用途地域分類コード | 11 |
| `prefecture` | string | 都道府県名 | 東京都 |
| `city_code` | string | 市区町村コード | 13101 |
| `city_name` | string | 市区町村名 | 千代田区 |
| `use_area_ja` | string | 用途地域名 | 商業地域 |
| `u_floor_area_ratio_ja` | string | 容積率 | 800% |
| `u_building_coverage_ratio_ja` | string | 建蔽率 | 80% |
| `decision_date` | string | 告示日 | — |
| `decision_classification` | string | 決定区分 | — |
| `decision_maker` | string | 決定主体 | — |

---

### 2.5 XKT006 — 学校施設

**用途**: 学校（小中高等学校等）の位置情報ポイントを取得

| 項目 | 値 |
|------|-----|
| URL | `https://www.reinfolib.mlit.go.jp/ex-api/external/XKT006` |
| メソッド | GET |
| ズームレベル | 13-15 |

#### パラメータ

| パラメータ | 型 | 必須 | 有効値 | 説明 |
|-----------|-----|------|--------|------|
| `response_format` | string | Yes | `geojson`, `pbf` | レスポンス形式 |
| `z` | integer | Yes | 13-15 | ズームレベル |
| `x` | integer | Yes | — | タイルX座標 |
| `y` | integer | Yes | — | タイルY座標 |

#### レスポンスプロパティ

| フィールド | 型 | 説明 | 例 |
|-----------|-----|------|-----|
| `P29_001` | string | 行政区域コード | 13101 |
| `P29_002` | string | 学校コード | B114220520022 |
| `P29_003` | integer | 学校分類コード | 16001 |
| `P29_003_name_ja` | string | 学校分類名 | 小学校 |
| `P29_004_ja` | string | 学校名 | 千代田区立九段小学校 |
| `P29_005_ja` | string | 所在地 | 東京都千代田区三番町16 |
| `P29_006` | integer | 設置者コード | 3 |
| `P29_007` | integer | 廃校分類 | 1 |

---

### 2.6 XKT010 — 医療機関

**用途**: 医療機関（病院・診療所）の位置情報ポイントを取得

| 項目 | 値 |
|------|-----|
| URL | `https://www.reinfolib.mlit.go.jp/ex-api/external/XKT010` |
| メソッド | GET |
| ズームレベル | 13-15 |

#### パラメータ

| パラメータ | 型 | 必須 | 有効値 | 説明 |
|-----------|-----|------|--------|------|
| `response_format` | string | Yes | `geojson`, `pbf` | レスポンス形式 |
| `z` | integer | Yes | 13-15 | ズームレベル |
| `x` | integer | Yes | — | タイルX座標 |
| `y` | integer | Yes | — | タイルY座標 |

#### レスポンスプロパティ

| フィールド | 型 | 説明 | 例 |
|-----------|-----|------|-----|
| `P04_001` | integer | 施設分類コード | — |
| `P04_001_name_ja` | string | 施設分類名 | 病院 |
| `P04_002_ja` | string | 施設名 | 東京大学医学部附属病院 |
| `P04_003_ja` | string | 所在地 | 東京都文京区本郷7-3-1 |
| `P04_004` - `P04_006` | string | 診療科目1-3 | 内科 |
| `medical_subject_ja` | string | 診療科目（結合） | 内科,外科 |
| `P04_007` | integer | 開設者分類 | — |
| `P04_008` | integer | 病床数 | 1217 |
| `P04_009` | integer | 救急指定 | 9=該当なし |
| `P04_010` | integer | 災害拠点病院 | 9=該当なし |

---

### 2.7 XKT016 — 災害危険区域

**用途**: 災害危険区域ポリゴンデータ（液状化・洪水等を包含）

| 項目 | 値 |
|------|-----|
| URL | `https://www.reinfolib.mlit.go.jp/ex-api/external/XKT016` |
| メソッド | GET |
| ズームレベル | 11-15 |

#### パラメータ

| パラメータ | 型 | 必須 | 有効値 | 説明 |
|-----------|-----|------|--------|------|
| `response_format` | string | Yes | `geojson`, `pbf` | レスポンス形式 |
| `z` | integer | Yes | 11-15 | ズームレベル |
| `x` | integer | Yes | — | タイルX座標 |
| `y` | integer | Yes | — | タイルY座標 |
| `administrativeAreaCode` | string | No | 5桁コード（カンマ区切り可） | 行政区域コード |

#### レスポンスプロパティ（A48 コード体系）

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `prefecture_name` | string | 都道府県名 |
| `city_name` | string | 市区町村名 |
| `region_name` | string | 区域名称 |
| `address` | string | 所在地 |
| `public_notice_date` | string | 告示日 |
| `landslide_area` | string | 区域面積 |

---

### 2.8 XKT022 — 急傾斜地崩壊危険区域

**用途**: 急傾斜地の崩壊危険区域ポリゴンデータを取得

| 項目 | 値 |
|------|-----|
| URL | `https://www.reinfolib.mlit.go.jp/ex-api/external/XKT022` |
| メソッド | GET |
| ズームレベル | 11-15 |

#### パラメータ

| パラメータ | 型 | 必須 | 有効値 | 説明 |
|-----------|-----|------|--------|------|
| `response_format` | string | Yes | `geojson`, `pbf` | レスポンス形式 |
| `z` | integer | Yes | 11-15 | ズームレベル |
| `x` | integer | Yes | — | タイルX座標 |
| `y` | integer | Yes | — | タイルY座標 |
| `prefectureCode` | string | No | 2桁コード（カンマ区切り可） | 都道府県コード |
| `administrativeAreaCode` | string | No | 5桁コード（カンマ区切り可） | 行政区域コード |

#### レスポンスプロパティ

| フィールド | 型 | 説明 | 例 |
|-----------|-----|------|-----|
| `prefecture_code` | string | 都道府県コード | 13 |
| `group_code` | string | 市区町村コード | 13101 |
| `city_name` | string | 市区町村名 | 千代田区 |
| `region_name` | string | 区域名称 | — |
| `address` | string | 所在地 | — |
| `public_notice_date` | string | 告示日 | 1984/03/31 |
| `landslide_area` | string | 区域面積 | 2.6(ha) |

---

## 3. タイル座標変換

### 3.1 BBox → タイル座標の変換式

```
lat_rad = lat * π / 180
n = 2^z
x = floor((lng + 180) / 360 * n)
y = floor((1 - ln(tan(lat_rad) + 1/cos(lat_rad)) / π) / 2 * n)
```

### 3.2 BBox カバレッジ計算

1つの bbox に対して複数タイルをカバーする必要がある:

```
x_min = tile_x(bbox.west, z)
x_max = tile_x(bbox.east, z)
y_min = tile_y(bbox.north, z)  // 注: 北が小さいy値
y_max = tile_y(bbox.south, z)
```

全 `(x, y)` 組み合わせに対してリクエストを発行し、結果をマージする。

### 3.3 推奨ズームレベル

| ユースケース | 推奨 z | 理由 |
|-------------|--------|------|
| 広域概観 | 11-12 | 市区町村レベルの概要 |
| 標準閲覧 | 13-14 | 地点レベルの詳細 |
| 詳細分析 | 15 | 最大解像度、タイル数増大に注意 |

---

## 4. Rust 実装時の型マッピング

### 4.1 共通パラメータ型

```rust
/// タイルベースAPI共通パラメータ
pub struct TileRequest {
    pub response_format: ResponseFormat,
    pub z: u8,      // 11-15
    pub x: u32,
    pub y: u32,
}

pub enum ResponseFormat {
    GeoJson,
    Pbf,
}
```

### 4.2 エンドポイント固有パラメータ

```rust
/// XPT001 追加パラメータ
pub struct TransactionPriceParams {
    pub from: String,       // "YYYYN" format
    pub to: String,         // "YYYYN" format
    pub price_classification: Option<PriceClassification>,
    pub land_type_code: Option<Vec<LandTypeCode>>,
}

/// XPT002 追加パラメータ
pub struct LandPriceParams {
    pub year: u16,
    pub price_classification: Option<u8>,  // 0 or 1
    pub use_category_code: Option<Vec<String>>,
}
```

### 4.3 レスポンス型方針

- 全エンドポイントのレスポンスは `serde_json::Value` として受信
- 必要なフィールドのみ型付き構造体にデシリアライズ
- 不明フィールドは `#[serde(flatten)] pub extra: HashMap<String, Value>` で保持

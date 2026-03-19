# 公的地理空間データソース調査レポート

> **調査日:** 2026-03-19
> **目的:** 不動産投資データビジュアライザーで使用する GeoJSON データについて、自作ではなく国土地理院・国交省等の公的機関が公開しているデータを調査する

---

## 目次

1. [調査背景](#1-調査背景)
2. [データソース一覧](#2-データソース一覧)
3. [国土数値情報ダウンロードサービス（詳細）](#3-国土数値情報ダウンロードサービス)
4. [不動産情報ライブラリ API（詳細）](#4-不動産情報ライブラリ-api)
5. [ハザードマップポータルサイト（詳細）](#5-ハザードマップポータルサイト)
6. [東京都建設局 液状化予測データ（詳細）](#6-東京都建設局-液状化予測データ)
7. [東京都オープンデータカタログ](#7-東京都オープンデータカタログ)
8. [e-Stat API（政府統計）](#8-e-stat-api)
9. [MLIT Geospatial MCP サーバー](#9-mlit-geospatial-mcp-サーバー)
10. [e-Stat MCP サーバー](#10-e-stat-mcp-サーバー)
11. [レイヤー別 最適データソース対応表](#11-レイヤー別-最適データソース対応表)
12. [取引価格 CSV データ分析](#12-取引価格-csv-データ分析)
13. [推奨実装方針](#13-推奨実装方針)

---

## 1. 調査背景

本プロジェクトでは東京都心を対象に以下8レイヤーのデータを3Dマップ上に可視化する：

| # | レイヤー | 形状 | 用途 |
|---|---------|------|------|
| 1 | 地価公示 | Point | 公示地価の空間分布 |
| 2 | 用途地域 | Polygon | 都市計画区域の色分け |
| 3 | 液状化リスク | Polygon | 地盤リスクの3D extrusion |
| 4 | 洪水浸水想定区域 | Polygon | 浸水深の3D extrusion |
| 5 | 急傾斜地崩壊危険区域 | Polygon | 土砂災害リスク |
| 6 | 学校 | Point | 教育施設マーカー |
| 7 | 医療機関 | Point | 医療施設マーカー |
| 8 | 不動産取引価格 | Point | 取引価格ヒートマップ |

**方針**: GeoJSON を自作するのではなく、公的機関が公開しているデータをそのまま、または最小限の変換で利用する。

---

## 2. データソース一覧

| ソース | 運営 | URL | 認証 | 形式 | 特徴 |
|--------|------|-----|------|------|------|
| **国土数値情報** | 国交省 | https://nlftp.mlit.go.jp/ksj/ | 不要 | GeoJSON/SHP/GML (ZIP) | 一括DL、CC BY 4.0 |
| **不動産情報ライブラリ API** | 国交省 | https://www.reinfolib.mlit.go.jp/ | 要APIキー | GeoJSON/PBF (タイル) | リアルタイム、最新データ |
| **ハザードマップポータル** | 国土地理院 | https://disaportal.gsi.go.jp/ | 不要 | ラスタPNGタイル | 防災データ網羅的 |
| **東京都建設局** | 東京都 | https://doboku.metro.tokyo.lg.jp/ | 不要 | Shapefile | 液状化予測詳細 |
| **東京都オープンデータ** | 東京都 | https://catalog.data.metro.tokyo.lg.jp/ | 不要 | GeoJSON | 区単位、限定的 |
| **e-Stat API** | 総務省 | https://www.e-stat.go.jp/ | 要appId | JSON/CSV | 統計データ（非地理） |
| **G空間情報センター** | 一般社団法人 | https://www.geospatial.jp/ | 不要 | 各種 | 横断検索ポータル |

---

## 3. 国土数値情報ダウンロードサービス

### 概要

- **URL**: https://nlftp.mlit.go.jp/ksj/
- **ライセンス**: CC BY 4.0（出典明記で自由利用可）
- **認証**: **不要**（ダウンロード自由）
- **フォーマット**: ZIP 内に GeoJSON + Shapefile + GML を同梱

### API でのダウンロードURL取得

```
http://nlftp.mlit.go.jp/ksj/api/1.0b/index.php/app/getKSJURL.xml
  ?appId=ksjapibeta1
  &lang=J
  &dataformat=1
  &identifier={CODE}
  &prefCode={PREF}
  &fiscalyear={YEAR}
```

レスポンスXML内の `<zipFileUrl>` にダウンロードURLが含まれる。

### ダウンロードURL パターン（推定）

```
https://nlftp.mlit.go.jp/ksj/gml/data/{CODE}/{CODE}-{YY}/{CODE}-{YY}_{PREF}_GML.zip
```

### 利用可能データセット（東京都 = 都道府県コード 13）

#### L01: 地価公示（Point）

| 項目 | 値 |
|------|-----|
| データページ | https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-L01-2024.html |
| 東京ファイル | `L01-24_13_GML.zip`（1.93MB） |
| 最新年 | 2024 |
| 主要属性 | `L01_006`（公示価格 円/㎡）、住所、地目 |
| 座標系 | JGD2011 / WGS84 |

#### A29: 用途地域（Polygon）

| 項目 | 値 |
|------|-----|
| データページ | https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A29.html |
| 東京ファイル | `A29-11_13_GML.zip`（6.51MB） |
| 最新年 | **2011**（やや古い） |
| 主要属性 | `youto`（用途地域名）、`youto_code`、容積率、建ぺい率 |
| 備考 | reinfolib API（XKT002）の方が新しいデータ |

#### A31: 洪水浸水想定区域（Polygon）

| 項目 | 値 |
|------|-----|
| データページ | https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A31-v2_1.html |
| 東京ファイル | `A31-12_13_GML.zip`（2.53MB） |
| 最新年 | 2012（東京）、他県は2024年まであり |
| 主要属性 | 浸水深ランク、想定条件、河川名 |
| 備考 | v4.0 (2024) 更新あるが東京は河川単位で分散 |

#### A47: 急傾斜地崩壊危険区域（Polygon）

| 項目 | 値 |
|------|-----|
| データページ | https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A47-v1_0.html |
| 東京ファイル | `A47-20_13_GML.zip`（50KB） |
| 最新年 | 2020 |
| 主要属性 | 区域名、指定年月日 |
| 備考 | 東京都心にはほぼ該当なし（山間部中心） |

#### P29: 学校（Point）

| 項目 | 値 |
|------|-----|
| データページ | https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-P29-v2_0.html |
| 東京ファイル | `P29-21_13_GML.zip`（0.49MB） |
| 最新年 | 2021 |
| 主要属性 | `P29_004`（学校名）、学校種別、所在地 |
| 備考 | 小中高校を含む |

#### P04: 医療機関（Point）

| 項目 | 値 |
|------|-----|
| データページ | https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-P04-v2_1.html |
| 東京ファイル | `P04-14_13_GML.zip`（2.29MB） |
| 最新年 | 2020 |
| 主要属性 | `P04_002`（施設名）、施設種別、病床数 |
| 備考 | 病院・診療所・歯科を含む |

### ダウンロード時の注意

- ZIP ファイル内のファイル名・構成はデータセットにより異なる
- GeoJSON は通常 `.geojson` 拡張子で同梱（一部は GML のみの場合あり）
- 座標系は原則 JGD2011（≒ WGS84、MapLibre でそのまま使用可）

---

## 4. 不動産情報ライブラリ API

### 概要

- **ベースURL**: `https://www.reinfolib.mlit.go.jp/ex-api/external/{ENDPOINT_ID}`
- **認証**: `Ocp-Apim-Subscription-Key` ヘッダーにAPIキーを付与
- **申請**: https://www.reinfolib.mlit.go.jp/ から無料申請（発行まで数日）
- **レスポンス**: GeoJSON / PBF（エンドポイントによる）
- **ステータス**: **APIキー申請中**（2026-03-18 申請済み）

### タイルベースエンドポイント（GeoJSON/PBF）

共通パラメータ:
```
?response_format=geojson&z={zoom}&x={tile_x}&y={tile_y}
```
- `z`: ズームレベル（通常 11-15）
- `x`, `y`: XYZスリッピーマップタイル座標

#### 価格・取引データ

| エンドポイント | 内容 | 形式 |
|-------------|------|------|
| **XIT001** | 不動産取引価格・成約価格情報 | JSON（非タイル） |
| **XIT002** | 都道府県別市区町村一覧 | JSON（非タイル） |
| **XPT001** | 取引価格ポイント（地図タイル） | GeoJSON / PBF |
| **XPT002** | 地価公示ポイント | GeoJSON / PBF |
| **XCT001** | 不動産鑑定評価書情報 | GeoJSON / PBF |

#### 都市計画・用途地域

| エンドポイント | 内容 | 形式 |
|-------------|------|------|
| **XKT001** | 都市計画区域・区域区分 | GeoJSON / PBF |
| **XKT002** | 用途地域 | GeoJSON / PBF |
| **XKT003** | 立地適正化計画区域 | GeoJSON / PBF |
| **XKT023** | 地区計画区域 | GeoJSON / PBF |
| **XKT024** | 高度利用地区 | GeoJSON / PBF |

#### 施設データ

| エンドポイント | 内容 | 形式 |
|-------------|------|------|
| **XKT004** | 小学校区域（通学区域） | GeoJSON / PBF |
| **XKT005** | 中学校区域（通学区域） | GeoJSON / PBF |
| **XKT006** | 学校施設位置 | GeoJSON / PBF |
| **XKT007** | 保育園・幼稚園 | GeoJSON / PBF |
| **XKT010** | 医療機関 | GeoJSON / PBF |
| **XKT011** | 福祉施設 | GeoJSON / PBF |
| **XKT015** | 駅別乗降客数 | GeoJSON / PBF |
| **XKT017** | 図書館 | GeoJSON / PBF |
| **XKT018** | 市区町村役所 | GeoJSON / PBF |

#### 防災データ

| エンドポイント | 内容 | 形式 |
|-------------|------|------|
| **XKT014** | 防火・準防火地域 | GeoJSON / PBF |
| **XKT016** | 災害危険区域 | GeoJSON / PBF |
| **XKT020** | 大規模盛土造成地マップ | GeoJSON / PBF |
| **XKT021** | 地すべり防止区域 | GeoJSON / PBF |
| **XKT022** | 急傾斜地崩壊危険区域 | GeoJSON / PBF |

### リクエスト例

```bash
curl -H "Ocp-Apim-Subscription-Key: YOUR_KEY" \
  "https://www.reinfolib.mlit.go.jp/ex-api/external/XKT002?response_format=geojson&z=14&x=14548&y=6451"
```

### 現在のバックエンド実装との対応

| フロントエンドID | バックエンドマッピング | reinfolib EP |
|----------------|---------------------|-------------|
| `landprice` | `XPT002` | 地価公示 |
| `zoning` | `XKT002` | 用途地域 |
| `liquefaction` | `XKT025` | （存在しない可能性あり — 要確認） |
| `flood` | `XKT026` | （存在しない可能性あり — 要確認） |
| `steep_slope` | `XKT022` | 急傾斜地崩壊危険区域 |
| `schools` | `XKT006` | 学校施設 |
| `medical` | `XKT010` | 医療機関 |

> **注意**: `XKT025`（液状化）と `XKT026`（洪水）はAPI仕様書で確認できていない。正しくは `XKT016`（災害危険区域）に統合されている可能性がある。APIキー取得後に実際のレスポンスを確認する必要がある。

---

## 5. ハザードマップポータルサイト

### 概要

- **URL**: https://disaportal.gsi.go.jp/
- **運営**: 国土地理院（国交省）
- **認証**: **不要**
- **形式**: **ラスタ PNG タイルのみ**（ベクターデータなし）

### タイルURL パターン

| データ | URL パターン |
|--------|-------------|
| 洪水浸水（最大規模） | `https://disaportaldata.gsi.go.jp/raster/01_flood_l2_shinsuishin_data/{z}/{x}/{y}.png` |
| 洪水浸水（計画規模） | `https://disaportaldata.gsi.go.jp/raster/01_flood_l1_shinsuishin_newlegend_data/{z}/{x}/{y}.png` |
| 急傾斜地崩壊危険区域 | `https://disaportaldata.gsi.go.jp/raster/05_kyukeishakeikaikuiki/{z}/{x}/{y}.png` |
| 土石流危険区域 | `https://disaportaldata.gsi.go.jp/raster/05_dosekiryukeikaikuiki/{z}/{x}/{y}.png` |
| 地すべり危険区域 | `https://disaportaldata.gsi.go.jp/raster/05_jisuberikeikaikuiki/{z}/{x}/{y}.png` |
| 液状化（東京都） | `https://disaportaldata.gsi.go.jp/raster/ekijouka_ken_13_tokyo/{z}/{x}/{y}.png` |

### 利用上の制約

- **GeoJSON ではない**ため、MapLibre の fill-extrusion（3D）レイヤーには使えない
- ラスタオーバーレイとしてのみ利用可能（`raster` ソースタイプ）
- 属性情報（浸水深の数値等）を取得できないため、スコアカード表示には不向き
- ズームレベル 2-17 対応

### 活用方法

MapLibre のラスタレイヤーとして重ねることは可能：
```typescript
// ラスタタイルソースとして追加
map.addSource('flood-raster', {
  type: 'raster',
  tiles: ['https://disaportaldata.gsi.go.jp/raster/01_flood_l2_shinsuishin_data/{z}/{x}/{y}.png'],
  tileSize: 256,
});
```

ただし、3D fill-extrusion やプロパティベースの色分けができないため、ベクターデータ（国土数値情報 or reinfolib API）が取得できる場合はそちらを優先すべき。

---

## 6. 東京都建設局 液状化予測データ

### 概要

- **URL**: https://doboku.metro.tokyo.lg.jp/start/03-jyouhou/ekijyouka/
- **ライセンス**: CC BY 2.1 JP
- **認証**: **不要**
- **形式**: **Shapefile のみ**（GeoJSON 非対応）

### 利用可能データセット

| ファイル | 内容 |
|---------|------|
| `shp/PL分布図.zip` | PL値（液状化指標）分布 |
| `shp/地下水位分布図.zip` | 地下水位分布 |
| `shp/液状化履歴図.zip` | 過去の液状化発生履歴 |
| `shp/埋立材区分図.zip` | 埋立地の材料区分 |

### 変換方法

Shapefile → GeoJSON 変換が必要：

```bash
# ogr2ogr（GDAL）を使用
ogr2ogr -f GeoJSON output.geojson input.shp -t_srs EPSG:4326

# Python（Fiona + Shapely）
import fiona
import json
with fiona.open("input.shp") as src:
    geojson = {"type": "FeatureCollection", "features": list(src)}
```

### 評価

- 東京都の液状化データとしては**最も詳細**
- ただし Shapefile → GeoJSON 変換が必要
- 国土数値情報には液状化の GeoJSON がないため、このソースが最有力
- 一度変換すれば `backend/data/` に配置して静的配信可能

---

## 7. 東京都オープンデータカタログ

- **URL**: https://catalog.data.metro.tokyo.lg.jp/dataset?res_format=GeoJSON
- 約40件の GeoJSON データセット
- 主に区単位（港区、品川区、目黒区等）の防災・施設データ
- 本プロジェクトのレイヤーを網羅するには不十分
- 補助データとして部分的に活用可能

---

## 8. e-Stat API

### 概要

- **URL**: https://www.e-stat.go.jp/
- **認証**: `appId` パラメータ（`.env` に設定済み）
- **形式**: JSON / CSV
- **特徴**: 統計データ中心（地理座標は限定的）

### 不動産関連で利用可能なデータ

#### 住宅・土地統計調査（2023年）

- **統計ID**: 住宅・土地統計調査（5年ごと）
- **東京ワード別**: 6,672データセット（1998年調査ベース、2024年4月更新）
- **内容**:
  - 都市計画区域別の25分類
  - 世帯構成タイプ別
  - 住宅所有パターン
  - 建築年代別7カテゴリ
  - 建物タイプ4分類
  - 構造2分類
  - 階数帯2バンド
  - 都市圏距離帯3バンド

#### MLIT 建築着工統計

- 43都道府県別データセット（2026年1月更新）
- 建築件数、延床面積、工事費見積
- 建築主体別分類

#### 国勢調査（2020年）

- 66市区町村別データセット（2024年9月更新）
- 産業・職業別の就業統計

### API エンドポイント

```
https://api.e-stat.go.jp/rest/3.0/app/json/getStatsData
  ?appId={APP_ID}
  &statsDataId={DATASET_ID}
  &limit=100000
```

### 評価

- **地理座標を含まない**統計データが中心
- マップ上のGeoJSONレイヤーとしては直接使えない
- スコアカードの補助情報（区平均価格、人口統計等）として活用可能
- 市区町村コードで国土数値情報のポリゴンと結合すれば空間統計化も可能

---

## 9. MLIT Geospatial MCP サーバー

### 概要

- **リポジトリ**: https://github.com/chirikuuka/mlit-geospatial-mcp
- **ツール**: `get_multi_api` — 30種の政府APIに統一アクセス

### 対応API一覧（30種）

| カテゴリ | 対象API |
|---------|--------|
| 不動産 | 取引価格、鑑定評価 |
| 都市計画 | 用途地域、土地利用、防火地域、地区計画、高度利用地区 |
| 教育施設 | 小学校、中学校、一般学校、保育園、幼稚園 |
| 公共施設 | 医療機関、福祉施設（3段階分類）、図書館、市区町村役場 |
| 交通 | 鉄道駅（乗降客数付） |
| 災害リスク | 指定災害区域、液状化傾向、最大規模洪水、高潮区域、津波浸水、土砂災害警戒、地すべり防止、急傾斜地崩壊（計10種） |
| 人口 | 将来人口推計メッシュ（250m解像度）、人口集中地区 |

### パラメータ

- `target_apis`: 配列で対象APIを指定
- 距離半径、価格分類、時期範囲、言語、行政区域コード、都道府県コード等 16種のフィルタ
- レスポンス: 標準化GeoJSON + 不動産情報ライブラリの地図URL
- ファイル保存機能あり（`save_file` パラメータ）

### 評価

- Claude Code や他のMCPクライアントからデータ取得に使える
- ただし本プロジェクトのバックエンドAPIとして直接組み込むのは不適切
- 調査・データ探索ツールとして有用

---

## 10. e-Stat MCP サーバー

### 概要

- **リポジトリ**: https://github.com/cygkichi/estat-mcp-server
- **ツール**: e-Stat REST API のMCPラッパー

### 利用可能な機能

- 統計データの検索・取得
- メタ情報（統計コード、カテゴリ等）の検索
- CSV/JSON 形式でのダウンロード

### 評価

- e-Stat API を直接叩くより使いやすい
- ただし返すデータは統計値（非地理）が中心
- 補助データ取得ツールとして位置付け

---

## 11. レイヤー別 最適データソース対応表

### 即座に利用可能（APIキー不要）

| レイヤー | 最適ソース | コード | 形式 | サイズ | 鮮度 |
|---------|-----------|--------|------|--------|------|
| **地価公示** | 国土数値情報 | L01 | GeoJSON (ZIP) | 1.93MB | 2024年 |
| **用途地域** | 国土数値情報 | A29 | GeoJSON (ZIP) | 6.51MB | 2011年 |
| **洪水浸水** | 国土数値情報 | A31 | GeoJSON (ZIP) | 2.53MB | 2012年 |
| **急傾斜地** | 国土数値情報 | A47 | GeoJSON (ZIP) | 50KB | 2020年 |
| **学校** | 国土数値情報 | P29 | GeoJSON (ZIP) | 0.49MB | 2021年 |
| **医療機関** | 国土数値情報 | P04 | GeoJSON (ZIP) | 2.29MB | 2020年 |

### APIキー取得後に切り替え可能

| レイヤー | ソース | エンドポイント | メリット |
|---------|--------|-------------|---------|
| 地価公示 | reinfolib API | XPT002 | 最新年データ、リアルタイム |
| 用途地域 | reinfolib API | XKT002 | **2011年→最新**（大幅改善） |
| 洪水等災害 | reinfolib API | XKT016 | 統合的な災害データ |
| 急傾斜地 | reinfolib API | XKT022 | 最新データ |
| 学校 | reinfolib API | XKT006 | 最新データ |
| 医療機関 | reinfolib API | XKT010 | 最新データ |
| 取引価格 | reinfolib API | XPT001 | **新規レイヤー追加可** |

### 特殊対応が必要

| レイヤー | 状況 | 対応案 |
|---------|------|--------|
| **液状化リスク** | 国土数値情報にGeoJSONなし | ① 東京都建設局 SHP → GeoJSON 変換、② ハザードマップPNGラスタ、③ reinfolib XKT016 |
| **取引価格** | ダウンロード CSV あり（59,056件） | 手元の `Tokyo_20244_20253.csv` を駅座標ベースでジオコーディング or reinfolib XPT001 |

---

## 12. 取引価格 CSV データ分析

### ファイル情報

- **ファイル**: `backend/data/Tokyo_20244_20253.csv`
- **エンコーディング**: CP932（Shift_JIS）
- **レコード数**: 約59,056件
- **期間**: 2024年第4四半期 〜 2025年第3四半期
- **ソース**: MLIT 不動産取引価格情報（直接ダウンロード）

### 主要カラム

| カラム名 | 内容 | 例 |
|---------|------|-----|
| `種類` | 取引種別 | 中古マンション等、宅地(土地と建物) |
| `地区名` | 地区 | 丸の内、銀座 |
| `市区町村名` | 市区町村 | 千代田区、中央区 |
| `最寄駅：名称` | 最寄り駅 | 東京、銀座 |
| `最寄駅：距離（分）` | 徒歩分数 | 5、10 |
| `取引価格（総額）` | 取引総額（円） | 85000000 |
| `面積（㎡）` | 土地/専有面積 | 65.5 |
| `間取り` | 部屋構成 | 2LDK |
| `建築年` | 建築年 | 令和3年 |
| `建物の構造` | 構造 | RC |
| `都市計画` | 用途地域 | 商業地域 |
| `取引時期` | 時期 | 2024年第4四半期 |

### ジオコーディング結果

`backend/data/station_coords.json` を使用した駅ベースジオコーディング：

- **マッチ率**: 99.8%（630駅中606駅で座標取得成功）
- **カバレッジ**: 58,960件/59,056件
- **アンマッチ**: 96件（17駅、主に郊外）
- **変換スクリプト**: `backend/lib/csv_to_geojson.py`

### 既存変換スクリプト（csv_to_geojson.py）

- 駅座標 + 徒歩距離（80m/分）から概算位置を算出
- ランダム角度 + ガウスノイズ（±30m）で散布
- シード固定（123）で再現性確保
- 出力: `backend/data/transactions.geojson`

> **方針確認**: このCSVはMLITが公式に公開しているダウンロードデータであり、データ自体は公的データ。ただし座標情報がないため、マップ表示にはジオコーディング（駅ベース or 住所ベース）が必要。reinfolib XPT001 が使えればAPIから座標付きデータを直接取得可能。

---

## 13. 推奨実装方針

### Phase 1: 国土数値情報 GeoJSON で即時開発（APIキー不要）

1. 国土数値情報から東京都の6データセット（L01, A29, A31, A47, P29, P04）をダウンロード
2. ZIP 内の GeoJSON を `backend/data/` に配置
3. バックエンドの `/api/area-data` を修正し、ローカル GeoJSON からバウンディングボックス内のフィーチャーをフィルタして返す
4. モックデータ（`mock_data.py`）を**実データ**に置き換え

**メリット**:
- APIキー待ちの間も実データで開発可能
- 公的データをそのまま使用（自作GeoJSONではない）
- オフラインでも動作

### Phase 2: reinfolib API 統合（APIキー取得後）

1. APIキー取得後、バックエンドのタイルフェッチを有効化
2. 用途地域（A29: 2011年 → XKT002: 最新）等、鮮度が問題のレイヤーをAPI経由に切り替え
3. 取引価格ポイント（XPT001）を新規レイヤーとして追加
4. 国土数値情報をフォールバックとして残す（API障害時）

### Phase 3: 追加データソース統合

1. 液状化データ: 東京都建設局 Shapefile → GeoJSON 変換
2. ハザードマップ: ラスタタイルオーバーレイとして追加（3Dではなく2Dオーバーレイ）
3. e-Stat 統計: スコアカードの補助情報（区平均価格、人口密度等）

### データフロー図

```
[国土数値情報 ZIP] → GeoJSON → backend/data/*.geojson
                                      ↓
                               [FastAPI Backend]
                                      ↓
[reinfolib API] → GeoJSON ──→ /api/area-data ──→ [Next.js Frontend]
   (Phase 2)                         ↓                    ↓
                              BBox フィルタ         MapLibre GL
                                                    3D Map
```

---

## 参考リンク

- [国土数値情報ダウンロードサイト](https://nlftp.mlit.go.jp/ksj/)
- [不動産情報ライブラリ API マニュアル](https://www.reinfolib.mlit.go.jp/help/apiManual/)
- [ハザードマップポータルサイト オープンデータ](https://disaportal.gsi.go.jp/hazardmap/copyright/opendata.html)
- [東京都建設局 液状化予測図](https://doboku.metro.tokyo.lg.jp/start/03-jyouhou/ekijyouka/)
- [東京都オープンデータカタログ](https://catalog.data.metro.tokyo.lg.jp/)
- [e-Stat API 仕様](https://www.e-stat.go.jp/api/api-info/e-stat-manual3-0)
- [G空間情報センター](https://www.geospatial.jp/)
- [mlit-geospatial-mcp (GitHub)](https://github.com/chirikuuka/mlit-geospatial-mcp)
- [estat-mcp-server (GitHub)](https://github.com/cygkichi/estat-mcp-server)
- [Reinfolib API 活用 (Qiita)](https://qiita.com/sanskruthiya/items/b9d3385aad544e6b58ad)
- [国土数値情報 API 解説 (GIS奮闘記)](https://www.gis-py.com/entry/kokudo-api)

# データインベントリ — 公式データソースマッピング

## 概要

本プロジェクトで使用する政府公開データの一覧。各データの所在、更新頻度、ダウンロード元をマッピングする。

## データ状態凡例

- ✅ 処理済み (GeoJSON 化、レイヤー表示可能)
- ⚠️ 未処理 (raw データあり、ETL 必要)
- ❌ 未取得 (ダウンロード必要)

---

## レイヤー別データ状態

### 静的レイヤー (Static GeoJSON → FlatGeobuf 移行対象)

| レイヤー | KSJ コード | 現在のスコープ | 状態 | raw ファイル | 処理済みファイル | 更新頻度 |
|----------|-----------|--------------|------|-------------|----------------|----------|
| 行政区画 | N03 | 東京23区 | ✅ | `data/raw/N03-2024*.zip` (672M) | `public/geojson/admin-boundary-tokyo.geojson` (11M) | 年1回 |
| DID (人口集中地区) | A00 | 全国+東京 | ✅ | `data/raw/others.zip` (region.shp) | `public/geojson/did-national.geojson` (29M), `did-tokyo.geojson` (1.4M) | 5年毎 (国勢調査) |
| 活断層 | F01 | 関東 | ✅ | `data/raw/others.zip` (fault.shp) | `public/geojson/fault-kanto.geojson` (74K) | 不定期 |
| 火山 | V05 | 関東 | ✅ | `data/raw/others.zip` (volcano.shp) | `public/geojson/volcano-kanto.geojson` (4.5K) | 不定期 |
| 地質 | G01 | 東京 | ✅ | `data/raw/geology.zip` (6.5M, Shapefile) | `public/geojson/geology-tokyo.geojson` (398K) | 数年毎 |
| 地形 | G03 | 東京 | ✅ | `data/raw/landform.zip` (23M, Shapefile) | `public/geojson/landform-tokyo.geojson` (1.1M) | 数年毎 |
| 土壌 | G02 | 東京 | ✅ | `data/raw/soil.zip` (17M, Shapefile) | `public/geojson/soil-tokyo.geojson` (986K) | 数年毎 |
| 地震ハザード | J-SHIS | 東京 | ✅ | — (API取得) | `public/geojson/jshis-seismic-tokyo.geojson` (161K) | 年1回 |
| 液状化 | PL | 東京 | ✅ | `data/raw/PL分布図.zip` (353K) | `public/geojson/pl-liquefaction-tokyo.geojson` (2.8M) | 不定期 |
| 鉄道路線 | N02 | 東京 | ✅ | `data/raw/N02-{05..24}_GML.zip` (17本) | `public/geojson/n02-railway-tokyo.geojson` (1.8M) | 年1回 |
| 浸水履歴 | — | 東京 | ✅ | `data/raw/1896_2019_sinsui_add24.zip` (6.9M) | `public/geojson/flood-history-tokyo.geojson` (4.5M) | 不定期 |

### API レイヤー (PostGIS 経由)

| レイヤー | KSJ コード | 現在のスコープ | 状態 | raw ファイル | 処理済みファイル | 更新頻度 |
|----------|-----------|--------------|------|-------------|----------------|----------|
| 地価 | L01 | 東京 (5年分) | ✅ | `data/raw/L01-{07..26}_GML.zip` (20本) | `data/geojson/l01-{2022..2026}-tokyo.geojson` | 年2回 |
| 用途地域 | A29 | 東京 | ✅ | `data/raw/A29-{11,19}_*_GML.zip` (94本) | `data/geojson/a29-zoning-tokyo.geojson` (51M) | 3-5年毎 |
| 洪水浸水想定 | A31b | 東京 | ✅ | `data/raw/A31b-24_10_*_GEOJSON.zip` (104本) | `data/geojson/a31b-flood-tokyo.geojson` (597M) | 年1回 |
| 急傾斜地 | A47 | 東京 | ✅ | `data/raw/A47-21_*_GML.zip` (45本) | `data/geojson/a47-steep-slope-tokyo.geojson` (54K) | 年1回 |
| 学校 | P29 | 東京 | ✅ | `data/raw/P29-{13,21,23}.zip` | `data/geojson/p29-schools-tokyo.geojson` (2.7M) | 年1回 |
| 医療施設 | P04 | 東京 | ✅ | `data/raw/P04-{14,20}_GML.zip` | `data/geojson/p04-medical-tokyo.geojson` (15M) | 年1回 |
| 駅 | S12 | 東京 | ✅ | `data/raw/S12-{12..24}_GML.zip` (12本) | `data/geojson/s12-stations-tokyo.geojson` (3.1M) | 年1回 |

### 未処理 raw データ (ETL 必要)

| データ | KSJ コード | スコープ | raw ファイル | サイズ | 用途 |
|--------|-----------|---------|-------------|--------|------|
| 津波浸水想定 | A33 | **全国** | `data/raw/A33-24_00_GEOJSON.zip` | 403M | tsunami レイヤー |
| 都市計画区域 | A16 | 都道府県別 | `data/raw/A16-15_GML.zip` | 44M | park / urban_plan レイヤー |
| 地盤沈下 | A40 | 多県 | `data/raw/A40-*_GML.zip` (52本) | ~950M | 補助情報 |
| 500mメッシュ | — | **全国** | `data/raw/500m_mesh_2024_GEOJSON.zip` | 510M | population_mesh レイヤー |
| 人口予測 | — | 東京 | `data/raw/Tokyo_20244_20253.csv` | 11M | population 補助データ |
| 土地取得 | P-Y2024 | — | `data/raw/P-Y2024-PRM-SHAPE.zip` | 6.9M | 参考データ |

### 未取得データ (Phase 2 以降でダウンロード)

| データ | KSJ コード | ダウンロード元 | 備考 |
|--------|-----------|--------------|------|
| 地すべり | A39 | https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A39.html | landslide レイヤー |
| 学区 | P11/P33 | https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-P11.html | school_district レイヤー |
| 公園 | P13 | https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-P13.html | park レイヤー |

---

## ダウンロード元マッピング

### 国土数値情報 (KSJ) — https://nlftp.mlit.go.jp/ksj/

| コード | データ名 | URL パターン |
|--------|---------|-------------|
| N03 | 行政区域 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N03-2024.html` |
| A00 | 人口集中地区 (DID) | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A00-v2_3.html` |
| A29 | 用途地域 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A29-v2_1.html` |
| A31b | 浸水想定区域 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A31b.html` |
| A33 | 津波浸水想定 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A33.html` |
| A39 | 地すべり防止区域 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A39.html` |
| A47 | 急傾斜地崩壊危険区域 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-A47.html` |
| F01 | 活断層 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-F01.html` (※ J-SHIS 推奨) |
| L01 | 地価公示 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-L01-2026.html` |
| N02 | 鉄道 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-N02-2024.html` |
| P04 | 医療施設 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-P04-v3_0.html` |
| P13 | 都市公園 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-P13.html` |
| P29 | 学校 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-P29.html` |
| P11 | 小学校区 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-P11.html` |
| S12 | 駅 | `https://nlftp.mlit.go.jp/ksj/gml/datalist/KsjTmplt-S12-2024.html` |

### その他データソース

| データ | ソース | URL |
|--------|--------|-----|
| J-SHIS 地震ハザード | 防災科研 | https://www.j-shis.bosai.go.jp/ |
| 液状化 (PL) | 東京都建設局 | https://doboku.metro.tokyo.lg.jp/ |
| 500mメッシュ人口 | e-Stat | https://www.e-stat.go.jp/ |
| 火山 (V05) | 気象庁 | https://www.data.jma.go.jp/svd/vois/data/tokyo/open-data/ |

---

## データサイズ概要

| カテゴリ | ファイル数 | サイズ |
|----------|----------|--------|
| `data/raw/` (未処理) | 377 | 6.7 GB |
| `data/geojson/` (処理済み作業用) | 14 | 1.5 GB |
| `public/geojson/` (本番用) | 12 | 108 MB |
| **合計** | **403** | **~8.3 GB** |

## 全国展開時の推定

| 項目 | 東京のみ | 全国47都道府県 | 倍率 |
|------|---------|---------------|------|
| PostGIS 行数 | ~30K | ~1M+ | 33x |
| 静的 GeoJSON | 108 MB | 3.5-4 GB | 35x |
| FlatGeobuf (圧縮後) | ~30 MB | ~1-1.5 GB | 40x |
| raw データ | 6.7 GB | ~20 GB | 3x |

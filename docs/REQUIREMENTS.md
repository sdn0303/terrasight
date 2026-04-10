# REQUIREMENTS.md — 不動産投資意思決定プラットフォーム

> Version: 1.0.0 | Updated: 2026-03-20
> Source: CEO Review, Eng Review, Design Review の全意思決定を統合

---

## 1. プロダクト概要

### 1.1 ミッション
日本全国の不動産投資判断に必要な情報（地価、防災リスク、都市計画、施設アクセス）を統合し、地図上のワンクリックで投資スコアを提示する意思決定プラットフォーム。

### 1.2 ターゲットユーザー
| ユーザー | ニーズ | Phase |
|---------|--------|-------|
| 個人不動産投資家 | エリア選定、物件比較、リスク評価 | Phase 1 |
| 不動産仲介業者 | デューデリジェンス支援、顧客への提案資料 | Phase 1-2 |
| 機関投資家 | ポートフォリオ分析、API経由のバルクデータ | Phase 3 |

### 1.3 ビジネスモデル
- **フリーミアム**: 基本閲覧無料 + プレミアム機能月額課金
- **B2B API**: 機関投資家・不動産テック企業向け従量課金

---

## 2. リポジトリ構造

```
repo-root/
├── services/
│   ├── frontend/              # Next.js 16 project
│   │   ├── src/
│   │   │   ├── app/           # App Router pages
│   │   │   ├── components/    # React components
│   │   │   ├── hooks/         # Custom hooks
│   │   │   └── lib/           # Utilities, API client, layer defs
│   │   ├── package.json
│   │   ├── next.config.ts
│   │   ├── tailwind.config.ts
│   │   └── Dockerfile
│   └── backend/               # Rust Axum project
│       ├── src/
│       │   ├── main.rs
│       │   ├── routes/        # Axum route handlers
│       │   ├── services/      # Business logic (scoring, etc.)
│       │   └── models/        # DB models, GeoJSON types
│       ├── migrations/        # sqlx PostGIS migrations
│       ├── Cargo.toml
│       ├── Cargo.lock
│       └── Dockerfile
├── data/
│   └── geojson/               # 国土数値情報 GeoJSON files (gitignored)
├── scripts/
│   ├── import_geojson.sh      # PostGIS data import
│   └── seed_dev_data.sql      # Development seed data
├── docs/
│   ├── REQUIREMENTS.md
│   ├── API_SPEC.md
│   ├── UIUX_SPEC.md
│   ├── designs/
│   ├── plans/
│   └── research/
├── docker-compose.yml
├── TODOS.md
├── .env.example
└── .gitignore
```

## 3. 技術スタック（確定済み意思決定）

| レイヤー | 技術 | 選定理由 |
|---------|------|---------|
| Frontend | Next.js 16 + TypeScript | App Router, RSC, Turbopack |
| Map | MapLibre GL JS + react-map-gl | OSS、Phase 2でMapbox検討 |
| UI | Tailwind CSS + framer-motion | ダークテーマ、CRTエフェクト |
| Charts | Recharts or visx | Sparkline、レーダーチャート |
| Backend | **Rust (Axum 0.8 + tokio)** | 空間計算性能、メモリ効率 |
| 空間計算 | geo / geozero / geojson crate | ネイティブ速度の座標演算 |
| DB Driver | sqlx (PostGIS via geo-types) | コンパイル時クエリ検証 |
| HTTP Client | reqwest | reinfolib / e-Stat API |
| Database | PostgreSQL 16 + PostGIS 3.4 | GiSTインデックスによる空間クエリ |
| Container | Docker Compose | postgis/postgis:16-3.4 |

### 2.1 却下された選択肢（理由付き）

| 選択肢 | 却下理由 |
|--------|---------|
| Python FastAPI | GIL制約、GeoJSON数GBのパース性能不足 |
| Yew (Rust WASM FE) | maplibre-rs が fill-extrusion/circle/heatmap/popup 未実装 |
| Mapbox GL JS (Phase 1) | Globe view不要、従量課金リスク。Phase 2で検討 |
| SQLite | 全国データ + 空間インデックスに非対応 |

---

## 4. 機能要件

### Phase 1（MVP — 5週間）

#### FR-1: 地図レイヤー表示
- **FR-1.1**: 7レイヤーのON/OFF切替（地価公示、用途地域、洪水浸水、急傾斜地、学校、医療機関、液状化）
- **FR-1.2**: レイヤーはカテゴリ別にグループ化（価格、都市計画、防災、施設）
- **FR-1.3**: 3Dダークマップ上にCARTO Dark Matter basemapで描画
- **FR-1.4**: 地価公示 → circle layer（cyan）
- **FR-1.5**: 用途地域 → fill layer（用途別カラーコード）
- **FR-1.6**: 防災リスク → fill-extrusion（リスクスコアに応じた高さ + 色）
- **FR-1.7**: 学校・医療機関 → symbol/circle marker
- **FR-1.8**: マップパン/ズーム時にビューポート内のデータを自動取得（debounce 300ms）

#### FR-2: 投資スコアリング
- **FR-2.1**: 地図上の任意の地点をクリック → 0-100の投資スコアを算出
- **FR-2.2**: スコア構成: 地価トレンド(0-25) + 防災リスク(0-25) + 施設アクセス(0-25) + 利回りポテンシャル(0-25)
- **FR-2.3**: 各コンポーネントスコアとその根拠を表示
- **FR-2.4**: スコアはゲージ（半円メーター）で可視化
- **FR-2.5**: 「参考値」であることを明示

#### FR-3: スコアカード（プロパティ詳細）
- **FR-3.1**: フィーチャークリック時に右パネルにスライドイン表示
- **FR-3.2**: 表示項目: 住所、㎡単価、用途地域、防災リスク、最寄り学校/医療機関
- **FR-3.3**: RAWデータ（GeoJSONプロパティ）のデバッグ表示（折りたたみ）

#### FR-4: エリア統計ダッシュボード
- **FR-4.1**: マップのビューポート連動でリアルタイム集計
- **FR-4.2**: 表示項目: 平均地価、物件数、リスクスコア分布、取引件数
- **FR-4.3**: マップを動かすたびに更新

#### FR-5: 比較分析
- **FR-5.1**: 2つのエリア/地点をサイドバイサイドで比較
- **FR-5.2**: レーダーチャートで5軸比較（地価、安全性、教育、医療、利回り）
- **FR-5.3**: 比較対象は地図上のピンで選択

#### FR-6: 地価推移チャート
- **FR-6.1**: スコアカード内にSparklineで過去5-10年の地価推移を表示
- **FR-6.2**: 国土数値情報L01の複数年度データを使用
- **FR-6.3**: 上昇/下降トレンドを色で区別（緑/赤）

#### FR-7: URL状態共有
- **FR-7.1**: 座標、ズーム、ピッチ、ベアリング、ON/OFFレイヤー、選択地点をURLクエリパラメータにエンコード
- **FR-7.2**: URLを開くと同じビューが再現される
- **FR-7.3**: ブラウザの戻る/進むで地図状態が復元

#### FR-8: CRT/ダークテーマUI
- **FR-8.1**: Shadowbroker風のCRTビネット + スキャンラインオーバーレイ
- **FR-8.2**: ダークテーマCSS変数（`--bg-primary: #0a0a0f` 等）
- **FR-8.3**: モノスペースフォント（Geist Mono）
- **FR-8.4**: 下部ステータスバー（座標、ズームレベル、デモモードバッジ）

### Phase 2（SaaS化）

#### FR-9: ユーザー認証
- **FR-9.1**: メール/パスワード or OAuth ログイン
- **FR-9.2**: JWT vs セッションの方針はPhase 1で決定

#### FR-10: ウォッチリスト + 通知
- **FR-10.1**: 条件保存（エリア + 価格帯 + リスク上限）
- **FR-10.2**: 条件に合致する新着データがあれば通知

#### FR-11: 月額課金
- **FR-11.1**: Stripe統合
- **FR-11.2**: フリー/プレミアムプランの機能制限

### Phase 3（成長）
- FR-12: AI投資予測モデル
- FR-13: ポートフォリオ管理
- FR-14: 機関投資家向けAPI
- FR-15: PDF/リンク共有レポート生成

---

## 5. 非機能要件

### NFR-1: パフォーマンス
| 指標 | 目標値 | 備考 |
|------|--------|------|
| bbox空間クエリ P99 | < 100ms | PostGIS GiSTインデックス前提 |
| 投資スコア算出 | < 500ms | 4コンポーネント並列計算 |
| フロントエンド初期ロード | < 3s (LCP) | MapLibre + basemap tiles |
| 地図パン後のデータ更新 | < 1s | debounce 300ms + API + レンダリング |
| 同時接続数 | 100+ | tokio非同期ランタイム |

### NFR-2: セキュリティ
- **NFR-2.1**: 全SQLクエリはパラメータバインド（`$1`, `$2`）。`format!()`禁止
- **NFR-2.2**: bbox面積制限（0.5度四方）でDoS防止
- **NFR-2.3**: CORS: 環境変数 `ALLOWED_ORIGINS` で制御
- **NFR-2.4**: レート制限: tower-governor（Phase 1内で実装）
- **NFR-2.5**: APIキー（reinfolib, e-Stat）はバックエンド限定、フロントに露出しない

### NFR-3: 可用性
- **NFR-3.1**: ヘルスチェックエンドポイント `/api/health`
- **NFR-3.2**: Docker healthcheck（curl ベース）
- **NFR-3.3**: PostGIS接続失敗時のグレースフルデグレード

### NFR-4: 可観測性
- **NFR-4.1**: 構造化ログ（`tracing` + `tracing-subscriber` with `env-filter`）
- **NFR-4.2**: RUST_LOG 環境変数でログレベル制御
- **NFR-4.3**: APIレスポンスタイム計測（tracing span）

### NFR-5: テスタビリティ
- **NFR-5.1**: バックエンド: `cargo test` + `#[sqlx::test]` 統合テスト
- **NFR-5.2**: フロントエンド: vitest
- **NFR-5.3**: 開発用seedデータ（東京駅周辺5-10行/テーブル）

### NFR-6: デプロイ
- **NFR-6.1**: Docker Compose で一括起動（db + backend + frontend）
- **NFR-6.2**: Rust バックエンドはマルチステージビルド（シングルバイナリ ~10MB）
- **NFR-6.3**: 環境変数: DATABASE_URL, REINFOLIB_API_KEY, ESTAT_APP_ID, RUST_LOG, ALLOWED_ORIGINS

---

## 6. データ要件

### 5.1 データソース（Phase 1）

| レイヤー | ソース | コード | 形式 | サイズ | 鮮度 |
|---------|--------|--------|------|--------|------|
| 地価公示 | 国土数値情報 | L01 | GeoJSON | 1.93MB/年/県 | 2020-2024（5年分） |
| 用途地域 | 国土数値情報 | A29 | GeoJSON | 6.51MB/県 | 2011年 |
| 洪水浸水 | 国土数値情報 | A31 | GeoJSON | 2.53MB/県 | 2012年 |
| 急傾斜地 | 国土数値情報 | A47 | GeoJSON | 50KB/県 | 2020年 |
| 学校 | 国土数値情報 | P29 | GeoJSON | 0.49MB/県 | 2021年 |
| 医療機関 | 国土数値情報 | P04 | GeoJSON | 2.29MB/県 | 2020年 |
| 液状化 | 東京都建設局 | — | Shapefile→変換 | — | — |

### 5.2 データソース（Phase 2 — reinfolib API切替）

| レイヤー | エンドポイント | 備考 |
|---------|-------------|------|
| 地価公示 | XPT002 | 最新年データ |
| 用途地域 | XKT002 | 2011→最新（大幅改善） |
| 学校 | XKT006 | 最新データ |
| 医療機関 | XKT010 | 最新データ |
| 取引価格ポイント | XPT001 | 新規レイヤー |

### 5.3 PostGIS スキーマ

```
land_prices     (id, price_per_sqm, address, land_use, year, geom:Point)
zoning          (id, zone_type, zone_code, floor_area_ratio, building_coverage, geom:MultiPolygon)
flood_risk      (id, depth_rank, river_name, geom:MultiPolygon)
steep_slope     (id, area_name, geom:MultiPolygon)
schools         (id, name, school_type, geom:Point)
medical_facilities (id, name, facility_type, bed_count, geom:Point)
```

全テーブルに `GIST(geom)` インデックス。

### 5.4 全国対応時のデータ量見積もり
- 地価公示: ~26万点（全国、5年分で130万行）
- 用途地域: ~数十万ポリゴン
- 医療機関: ~18万点
- 学校: ~3.5万点
- **受入基準**: bbox P99 < 100ms（首都圏4県で性能検証後、段階的に全国展開）

---

## 7. 制約事項

| 制約 | 内容 |
|------|------|
| reinfolib APIキー | 申請中（2026-03-18）。Phase 1は国土数値情報で代替 |
| 液状化データ | 国土数値情報にGeoJSONなし。東京都建設局SHP変換 or reinfolib XKT016で対応 |
| XKT025/026 | reinfolib APIに存在しない可能性。国土数値情報A31で代替 |
| デプロイ先 | 未定（Phase 2で決定） |
| 認証方式 | 未定（JWT vs セッション、Phase 1で方針決定） |

---

## 8. NOT in scope

| 項目 | 理由 |
|------|------|
| モバイルネイティブアプリ | Web first、レスポンシブで十分 |
| AI物件推薦 | Phase 3、まずルールベーススコアリング |
| 物件売買仲介 | 情報提供プラットフォームに集中 |
| リアルタイム価格更新 | 公的データは四半期更新 |
| 多言語対応 | 日本市場に集中 |
| ストリートビュー統合 | 工数対効果低 |
| 課金 / 認証 / 権限 | Phase 2 SaaS 化以降 |
| UI のピクセル単位レイアウト確定 | UIUX_SPEC.md に委譲 |
| raw データ取得元 URL の最終採用一覧 | 次フェーズ「データ再構築計画」で分解 |

---

## 9. 受け入れ条件 (target state, ward 対応フェーズ)

> 旧 `TERRASIGHT_SPEC_V1.md` §15 から統合。現状実装 (`prefecture | municipality`) から
> 全国 / ward 対応への移行時に満たすべき acceptance criteria を定義する。

### 9.1 機能受け入れ

1. 行政界クリックで `highlight / breadcrumb / stats refetch / popup close` が成立する (全 level 共通)
2. `prefecture / municipality / ward` の各 level を選択できる
3. `selected_prefecture` 系レイヤーは、選択時に都道府県全域表示へ切り替わる
4. `zoom < 9` で national fallback する

### 9.2 契約受け入れ

1. viewport 系 query は `map.getBounds()` の実 bbox を使う (live `viewState` からの近似再計算は禁止)
2. query key は debounced state のみを使う
3. static layer の batched path で duplicate fetch が起きない
4. `name` 非依存で UI が成立する (`wardName ?? cityName ?? prefName` で派生)
5. 東京都道府県コード `"13"` の hardcode を全国仕様として常設しない

### 9.3 E2E 受け入れ

最低限、以下を E2E で保証:

1. 行政界クリックで highlight が表示される
2. breadcrumb が期待階層で更新される
3. area stats が新しい `selectedArea.code` で再取得される
4. 既存 popup が閉じる
5. pan 中に refetch が連打されず、move end 後にのみ query される
6. `admin_boundary` の duplicate fetch が発生しない
7. missing source / invalid filter による console error が出ない

### 9.4 データ戦略 (canonical dataset 要件)

現行データは初期開発用モックであり、次フェーズで全国 canonical dataset を再構築する前提:

1. 47 都道府県をカバーする
2. 行政コードが安定している
3. 階層関係 (`parentCode`) が機械的に復元できる
4. layer ごとの geometry / property schema が固定である
5. バージョン管理可能である

詳細 (raw source 棚卸し、ETL 実装詳細、property マッピング) は次フェーズ「データ再構築計画」で分解する。

---

## 10. 用語集

| 用語 | 説明 |
|------|------|
| 国土数値情報 | 国交省が提供するCC BY 4.0の地理空間データ。GeoJSON/SHP形式で無料DL |
| reinfolib API | 国交省・不動産情報ライブラリのREST API。タイルベースGeoJSON。APIキー必要 |
| e-Stat API | 総務省・政府統計ポータルのAPI。統計データ（非地理）を提供 |
| PostGIS | PostgreSQLの空間拡張。GiSTインデックスによる高速空間クエリ |
| bbox | Bounding Box。南西・北東の緯度経度で定義される矩形範囲 |
| fill-extrusion | MapLibre/MapboxのGL層タイプ。ポリゴンを3D押し出しで表示 |
| CAGR | Compound Annual Growth Rate。年平均成長率 |
| GiST | Generalized Search Tree。PostGISの空間インデックス方式 |

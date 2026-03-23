# TODOS

> 最終更新: 2026-03-23 (Phase 1完了 🎉 — P0/P1/P1.5全完了、次はP2 SaaS化)

---

## P0 — 現フェーズで対応

_全P0タスク完了。_

---

## P1 — Phase 1内で対応

_全P1タスク完了。_

---

## P1.5 — Phase 1完成に向けた残作業

_全P1.5タスク完了。_ DB実データ投入(757K行) + API動作検証済み。

---

## P2 — SaaS化フェーズ

### CI/CDパイプライン
- **What**: GitHub Actions で clippy + cargo test + pnpm test + build + deploy
- **Why**: 手動テスト依存はスケールしない。PR毎の自動チェックが必要
- **Effort**: S | **Priority**: P2-high
- **Blocked by**: なし（今すぐ着手可能）

### 認証方針の決定
- **What**: JWT vs セッションベースの方向性を決定。Phase 2で実装
- **Why**: Axum の tower middleware 構成と AppState に影響
- **Effort**: S（決定のみ）| **Priority**: P2-high

### 全国対応の性能検証
- **What**: まず首都圏4県（東京+神奈川+埼玉+千葉）で性能テスト、bbox P99 < 100msを基準に
- **Why**: A31b洪水(643K features)投入後にクエリ性能が劣化する可能性。パーティショニングの必要性を検証
- **Effort**: M | **Priority**: P2-high

### 可観測性基盤（FE pino追加）
- **What**: FE側にpino構造化ログ追加。BE側はtracing + tracing-subscriber + telemetry crate実装済み
- **Effort**: XS | **Priority**: P2

### Mapbox GL JS 切替検討
- **What**: Phase 1はMapLibre維持。Globe view/terrain 3D/Mapbox Studioが必要になったら切替
- **Why**: 3箇所の変更で低リスク。従量課金はSaaS収益でカバー
- **Effort**: S | **Priority**: P2（トリガー待ち）

### UIUX_SPEC.md 更新
- **What**: CRT/Shadowbrokerテーマ参照をUrban Stratigraphyデザインシステムに更新
- **Why**: ステールなドキュメントは新エンジニアの混乱を招く。globals.cssとの乖離が拡大
- **Effort**: S (human 4h / CC 15min) | **Priority**: P2
- **File**: `docs/UIUX_SPEC.md`

### DESIGN.md 作成
- **What**: Urban Stratigraphyデザインシステムの公式ドキュメント（カラートークン、タイポグラフィ、コンポーネントパターン）
- **Why**: globals.cssにデザイントークンが定義済みだがドキュメント化されていない。24レイヤー拡大後、デザインレビューの基準が不明確
- **Effort**: S (human 4h / CC 15min) | **Priority**: P2
- **File**: `DESIGN.md`

---

## Completed (P1)

### ~~cargo test 統合テスト拡充（BE）~~ ✅
- axum-test による HTTP統合テスト11本追加。全5エンドポイント網羅

### ~~セキュリティ強化~~ ✅
- CORS明示設定 + レート制限（tower-governor IP-based token bucket）

### ~~XKT025/026 代替データ経路の確定~~ ✅
- 液状化→東京都建設局PL分布図、洪水→国土数値情報A31b、J-SHIS液状化データなし確定
- `scripts/convert-geodata.py` (geopandas, 15データセット), `docs/research/2026-03-23-data-inventory.md`

### ~~layers.ts の endpoint フィールド整理~~ ✅
- `source: "api" | "static"` に置換

### ~~L01 複数年度インポート手順~~ ✅
- `scripts/import-l01.py` (5年分 20,914行, dry-run確認済み)

### ~~reinfolib API モック/スタブ~~ ✅
- `ReinfolibDataSource` trait + `PostgisFallback` + `LiveReinfolib` stub + factory

### ~~J-SHIS API クライアント~~ ✅
- `jshis.rs` 3エンドポイント, wiremockテスト10本, 合計82テスト

### ~~J-SHIS → スコア計算への統合~~ ✅
- `ComputeScoreUsecase` に `Option<Arc<JshisClient>>` 注入、4因子リスク計算（flood=0.25, seismic=0.30, steep=0.15, ground_amp=0.30）

### ~~ReinfolibDataSource → AppState接続~~ ✅
- `&Config` 参照をcomposition rootに伝播、factory自動選択

### ~~変換済みGeoJSON → PostGISバルクインポートスクリプト~~ ✅
- `import-geojson.py` 9データセット736,703行対応、dry-run確認済み、マイグレーション追加

### ~~FE: 新レイヤー表示対応（液状化・地震動・鉄道）~~ ✅
- 3新レイヤーコンポーネント + layers.ts + CSS variables、24レイヤー体制（21→24）、134テスト通過

### ~~L01 実データ投入~~ ✅
- `import-l01.py` で5年分20,914行をPostGISに投入。ON CONFLICT (address, year) UPSERT対応

### ~~GeoJSON実データ投入~~ ✅
- `import-geojson.py` で9データセット736,504行を投入（flood 638K, medical 42K, zoning 21K, liquefaction 20K, schools 8K, railways 4K, stations 3K, steep 75, seismic 25）
- mixed geometry対応: railways/stations DDLを`geometry(Geometry, 4326)`に修正、非Polygon geometryフィルタ追加

### ~~レート制限500エラー修正~~ ✅
- `axum::serve`に`into_make_service_with_connect_info::<SocketAddr>()`を追加。PeerIpKeyExtractorがConnectInfoを取得できず全リクエスト500になる問題を修正

### ~~API動作検証（実データ）~~ ✅
- 全5エンドポイント（health, trend, score, stats, area-data）をPostGIS 757K行に対してE2E検証済み

## Completed (P0以前)

<details>
<summary>展開して表示</summary>

- ~~useMapData メモリリーク修正~~ ✅ — TanStack Query移行
- ~~MapLibre addSource エラーハンドリング~~ ✅ — PR1
- ~~WebGL Context Lost リカバリー~~ ✅ — PR1
- ~~レイヤートグル中の fetch レースコンディション~~ ✅ — PR1
- ~~page.tsx レイヤー宣言的レンダリングリファクタ~~ ✅ — PR1
- ~~Vitest テスト基盤構築（FE）~~ ✅ — PR2 (43テスト)
- ~~Rust Axum バックエンド移行~~ ✅ — Clean Architecture + 5 API endpoints
- ~~PostGIS seed データ作成~~ ✅ — 東京駅周辺41行

</details>

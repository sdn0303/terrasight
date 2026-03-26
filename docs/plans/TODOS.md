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

## P0.5 — データ正確性修正（Codex Audit 2026-03-27 + Eng Review）

> 参照: `docs/reviews/2026-03-27-frontend-wasm-backend-db-audit.md`
> **P1.6 UXクラフトの前に全件修正必須**

### X-01: flood depth_rank 契約不整合
- **What**: DB=smallint, Backend DTO=Option<String>, Zod=z.string(), FloodLayer rendering=文字列前提。canonical representation を数値に統一
- **Why**: 洪水レイヤーが誤描画/クラッシュする。P0
- **Effort**: S（CC: 15min）| **Priority**: P0
- **Validation**: integration test で `/api/area-data?layers=flood` の depth_rank 型を assert + FloodLayer style expression テスト

### X-02: WASM stats サイレント誤計算
- **What**: WASM partial load でも `ready=true` → 不足レイヤーを0で返す。直近は WASM stats 優先経路を無効化し `/api/stats` を canonical source に戻す
- **Why**: ユーザーは気づかず誤った統計を見る。P0
- **Effort**: S（CC: 15min）| **Priority**: P0
- **Validation**: `requiredLayers` 欠損時に `ready=false` テスト + API/WASM parity テスト

### DB-01: land_prices.zone_type 未投入
- **What**: `import_l01.py` が zone_type を INSERT しない → TLS z-score の前提崩れ。spatial join で zone_type 補完を実装
- **Why**: 地価スコアリングが dev seed と本データで結果が異なる。P0
- **Effort**: M（CC: 30min）| **Priority**: P0
- **Validation**: import 後 `SELECT count(*) FROM land_prices WHERE zone_type IS NULL` = 0

### DB-02: railways.station_name schema 不整合
- **What**: schema redesign で station_name 列が消失したが importer は依然 mapping。列を復元 or importer 修正
- **Why**: db-full-reset 時に railway import 失敗。P0
- **Effort**: S（CC: 10min）| **Priority**: P0
- **Validation**: `db-full-reset.sh` が railway import 含め全件成功

### X-04: selectedArea 配線
- **What**: `selectArea()` を呼ぶ call site が存在しない（dead code）。行政界クリック → `selectArea()` → AreaCard 表示までの配線実装
- **Why**: Explore drill-down が end-to-end で動作しない。モードマージの前提
- **Effort**: M（CC: 30min）| **Priority**: P1
- **Depends on**: admin_boundaries import 完了

### X-05: popupFields / schema / asset drift → LayerRegistry 統合
- **What**: layer 情報が layers.ts / schemas.ts / layer components / DESIGN.md に分散。単一 `LayerRegistry` に統合し build-time validation
- **Why**: popup 空表示、static layer 404 サイレント失敗の根因。モードマージ・spatial popup の前提
- **Effort**: M（CC: 30min）| **Priority**: P1
- **Validation**: 全 popupFields.key が schema に存在 + static asset 存在確認テスト

### X-06: bbox 近似 → map.getBounds() 化
- **What**: center + zoom 近似 → MapLibre `map.getBounds()` を single source に。query 用 viewport state を debounce 分離
- **Why**: pitch/bearing 変更時にデータ fetch 精度が落ちる + pan 中の高頻度再フェッチ
- **Effort**: S（CC: 15min）| **Priority**: P1
- **Validation**: pitch 変更時の getBounds() と近似 bbox の乖離テスト

---

## P1.6 — UXクラフトフェーズ（CEO Review 2026-03-27）

### page.tsx God Component リファクタ
- **What**: 336行の page.tsx を MapContainer / useMapInteraction hook / LayerRenderer に分割。page.tsx はレイアウトシェル（~30行）のみに
- **Why**: 全UX改善の構造的基盤。現状のモノリスはメンタルモデルの混乱原因。クロスストア直接操作（handleFeatureClick内のuseAnalysisStore.getState()）も解消
- **Pros**: UXの思考単位がコンポーネント単位になり、改善・テストが容易に
- **Cons**: 既存動作の一時的な回帰リスク（テストで担保）
- **Effort**: M（CC: 30min）| **Priority**: P1.6-critical
- **Depends on**: なし（最初に着手）

### モードシステム簡略化（3→2モード）
- **What**: Explore + Analyze を統合し、プログレッシブ・ディスクロージャーで1モードに。Compare は維持。フィーチャークリック時の自動モード切替を廃止
- **Why**: 3モードは認知負荷が高く、「使い方がわからない」原因。クリック→自動Analyze切替が最大のUX混乱ポイント
- **Pros**: エリアクリック→概要表示→フィーチャークリック→詳細展開の自然なフロー
- **Cons**: AnalyzePanel の内容をExplorePanel に統合する設計作業が必要
- **Effort**: L（CC: 1hr）| **Priority**: P1.6-critical
- **Depends on**: page.tsx リファクタ

### エラー状態の可視化
- **What**: (a) WASM spatial engine初期化失敗時のエラーバウンダリ+トースト (b) 地価レイヤーAPIエラー時のインラインメッセージ (c) ズーム不足時の説明オーバーレイ
- **Why**: 現状すべてサイレントフェイル。ユーザーは空のマップを見て何が起きたかわからない
- **Pros**: デバッグ容易、ユーザー体験向上
- **Cons**: エラーUI設計が必要（最小限で良い）
- **Effort**: S（CC: 15min）| **Priority**: P1.6-high
- **Depends on**: なし

### ガイド付き初回体験
- **What**: アプリ起動時に千代田区を中心に「安全性」テーマプリセットをアクティブ化。オンボーディングツールチップ表示
- **Why**: 現状の空マップ+「クリックして探索」は初見で何も伝えない。データの魅力を即座に見せる
- **Effort**: S（CC: 15min）| **Priority**: P1.6-medium
- **Depends on**: なし

### 空間ポップアップ（MapLibre Popup連携）
- **What**: PopupCard を画面中央固定からMapLibre Popupアンカーに移行。クリック位置に追従
- **Why**: 地図アプリの基本UX。現在のcenter-fixedは空間との対応関係が切れている
- **Effort**: S（CC: 15min）| **Priority**: P1.6-medium
- **Depends on**: なし

### レイヤーキュレーション（テーマカード大型化）
- **What**: 24レイヤーを4つのテーマ（安全性/利便性/価格/将来性）に整理。大型カードUIで直感的に選択
- **Why**: 24レイヤー羅列は圧倒的。テーマプリセットは既存だが小さいボタンで目立たない
- **Effort**: M（CC: 30min）| **Priority**: P1.6-medium
- **Depends on**: モードシステム簡略化

### パネルトランジション
- **What**: コンテキストパネルの内容変更時にframer-motion/View Transitions APIでアニメーション
- **Why**: 「機能的」と「クラフト品質」の差。状態変化をアニメーションで伝達
- **Effort**: M（CC: 30min）| **Priority**: P1.6-low
- **Depends on**: モードシステム簡略化

### デザインシステム実装
- **What**: DESIGN.md の色トークン・タイポグラフィをCSS変数として全UIに適用。text-cyan-400等の直接指定を置換
- **Why**: デザインシステムは文書上に存在するがコードに未反映。一貫性が品質の基盤
- **Effort**: M（CC: 30min）| **Priority**: P1.6-medium
- **Depends on**: なし（他タスクと並行可能）

---

## P2 — SaaS化フェーズ

### 密集地域のHex Grid集約（3D地価レイヤー）
- **What**: 銀座・新宿などの高密度エリアで、30mポリゴン正方形がオーバーラップしてz-fightingする問題をサーバーサイドHex Grid集約で解決
- **Why**: zoom 13+の高密度地域で3Dカラムが重なり、視覚的ノイズが発生。Design Review（2026-03-24）で特定
- **Pros**: 全ズームレベルでクリーンな可視化、データ可読性向上
- **Cons**: PostGIS集約クエリの新規実装が必要、API複雑性増加
- **Effort**: M | **Priority**: P2-medium
- **Depends on**: Phase 2 LOD戦略の策定

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

### FE↔BEスキーマ契約テスト
- **What**: バックエンドのRust応答型とFEのZodスキーマの一致を自動検証するCIチェック
- **Why**: Eng Review(2026-03-23)で3つのスキーマ不一致が発覚（composite_risk, depth_rank型, river_name nullable）。手動管理では再発する
- **Effort**: M | **Priority**: P2-medium
- **Blocked by**: CI/CDパイプライン

### LiveReinfolibスタブ実装
- **What**: `reinfolib_mock.rs` の5つのTODOメソッドを実装、MLIT APIとのライブ統合
- **Why**: 現在はPostGISフォールバックで全データ取得。SaaS化時にはAPIキー有効なユーザー向けにライブデータ提供が必要
- **Effort**: L | **Priority**: P2-low
- **Blocked by**: MLIT APIキー取得

---

## Completed (P2)

### ~~可観測性基盤（FE pino追加）~~ ✅
- pino構造化ログ追加。logger.ts（singleton, env制御）、api.tsにkyライフサイクルフック、map-view.tsxのconsole.*を置換。テスト4本追加（43→47）

### ~~Mapbox GL JS 切替検討~~ ✅
- `docs/plans/mapbox-migration-guide.md` 作成。26ファイルのMapLibreフットプリント調査、react-map-glが両対応のためimportパス変更+型1行で移行可能と確認。トリガー待ち

### ~~UIUX_SPEC.md 更新~~ ✅
- CRT/Shadowbroker参照を全削除。Urban Stratigraphyデザインシステムに統一、24レイヤー・5カテゴリ・実CSS変数値を反映。v1.0→v2.0

### ~~DESIGN.md 作成~~ ✅
- `DESIGN.md` 作成。24レイヤーカラートークン、ベースパレット、Geist Sans/Monoタイポグラフィ、shadcn/uiコンポーネントパターン、レイヤーカテゴリ、ダークモードテーマ仕様を文書化

### ~~FE↔BEスキーマ不一致修正~~ ✅
- Eng Review発見: `composite_risk` (avg_→削除), `depth_rank` (number→string+matchマップ), `river_name` (nullable化), `land_use` (nullable化)。schemas.ts + dashboard-stats.tsx + flood-layer.tsx + テストモック更新

### ~~ヘルスチェックタイムアウト追加~~ ✅
- `pg_health_repository.rs` に `tokio::time::timeout(3s)` 追加。DB障害時にヘルスチェックが無限ブロックする問題を修正

### ~~WebGLクリーンアップ修正~~ ✅
- `map-view.tsx` の handleLoad で登録した webglcontextlost/restored リスナーの removeEventListener 追加。空だったリカバリータイムアウトに triggerRepaint フォールバック実装

### ~~AbortController統合（ky + TanStack Query）~~ ✅
- 全5フック (use-area-data, use-stats, use-trend, use-score, use-health) で TanStack Query の signal を ky に転送。地図パン時の不要なHTTPリクエストがキャンセルされDB負荷を軽減

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

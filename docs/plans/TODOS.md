# TODOS

> 最終更新: 2026-03-22 (P0完了)

---

## P0 — 現フェーズで対応

_全P0タスク完了。_

---

## P1 — Phase 1内で対応

### ~~cargo test 統合テスト拡充（BE）~~ ✅
- **Completed**: axum-test による HTTP統合テスト11本追加。全5エンドポイント網羅
- lib.rs 抽出で `build_router()` をテストから呼び出し可能に
- テスト: health(1), area-data(5: 正常系3+異常系2), score(1), stats(1), trend(1), seed検証(2)
- DATABASE_URL未設定時は graceful skip（CIでDB不要時も安全）
- 合計39テスト（28 unit + 11 integration）、0 clippy warnings

### ~~セキュリティ強化~~ ✅
- **Completed**: CORS明示設定 + レート制限実装済み
- CORS: `ALLOWED_ORIGINS` env var → `CorsLayer` explicit origin whitelist（未設定時はpermissive dev mode）
- Rate Limit: `tower-governor` IP-based token bucket（`RATE_LIMIT_RPM=120`, `RATE_LIMIT_BURST=20`デフォルト）
- BBox範囲制限（0.5°制限）、入力バリデーション（BBox/Coord value objects）実装済み
- 28 cargo tests passing, 0 clippy warnings

### XKT025/026 代替データ経路の確定
- **What**: 液状化(XKT025)と洪水(XKT026)のreinfolib APIエンドポイントが存在しない可能性。代替ソースを確定させる
- **Why**: 現在のDEMO_MODEでは動作するが実データ移行時に破綻
- **Effort**: S | **Priority**: P1
- **対応案**: 液状化→東京都建設局SHP変換、洪水→国土数値情報A31のGeoJSON

### ~~layers.ts の endpoint フィールド整理~~ ✅
- **Completed**: PR1のレイヤー拡張リファクタで `endpoint` フィールド除去済み。`source: "api" | "static"` に置換

### L01 複数年度インポート手順
- **What**: Sparkline用に5年分（2020-2024）の地価公示データをインポートする手順を実装
- **Why**: 投資スコアの「地価トレンド」算出とSparkline表示に必要
- **Effort**: S | **Priority**: P1

---

## Completed

### ~~useMapData メモリリーク修正~~ ✅
- **Completed**: TanStack Query移行で解決済み
- `useMapData.ts` → `use-area-data.ts` (TanStack Query) にリファクタ。手動debounceRef不要に
- map-view.tsx / year-slider.tsx のtimerRefも `useEffect` cleanup済み

### ~~MapLibre addSource エラーハンドリング~~ ✅
- **Completed**: PR1 (feature/pr1-layer-expansion)
- try/catch wrapper added to map-view.tsx handleLoad

### ~~WebGL Context Lost リカバリー~~ ✅
- **Completed**: PR1 (feature/pr1-layer-expansion)
- webglcontextlost/restored event handlers + toast overlay + reload button

### ~~レイヤートグル中の fetch レースコンディション~~ ✅
- **Completed**: PR1 (feature/pr1-layer-expansion)
- Static layers use `if (!visible) return null` pattern — Source unmount cancels fetch. No stale data possible.

### ~~page.tsx レイヤー宣言的レンダリングリファクタ~~ ✅
- **Completed**: PR1 (feature/pr1-layer-expansion)
- Component registry pattern + source field in layers.ts + two loops (static/API)

### ~~Vitest テスト基盤構築（FE）~~ ✅
- **Completed**: PR2 (feature/pr2-vitest-foundation)
- 43テスト: layers.ts設定整合性(17) + map-store拡張(8) + ui-store(6) + 既存(12)
- layers.test.ts / map-store-extended.test.ts / ui-store.test.ts

### ~~Rust Axum バックエンド移行~~ ✅
- **Completed**: Clean Architecture実装済み (handler/usecase/domain/infra)
- 5 APIエンドポイント: health, area-data, score, stats, trend
- 25 cargo tests passing, clippy clean
- Workspace: 5 lib crates (telemetry, geo-math, db, api-core, mlit-client)
- PostGIS schema migration + GIST indexes on all geometry columns
- 全SQLパラメータバインド済み（$1, $2）、format!()なし

### ~~PostGIS seed データ作成~~ ✅
- **Completed**: migrations/20260322000001_seed_dev.sql
- 東京駅周辺: land_prices(15行/5年分), zoning(5), flood_risk(4), steep_slope(3), schools(8), medical(6)
- 冪等INSERT（WHERE NOT EXISTS）で再実行安全
- scripts/seed-dev.sh でワンコマンド投入

### reinfolib API モック/スタブ
- **What**: reinfolib_mock.rsでモックレスポンスを提供し、APIキー取得前の並行開発を可能にする
- **Why**: APIキー申請は2-4週かかる可能性。Phase 2の不動産取引価格レイヤー(#8)開発をブロックしない
- **Effort**: S (human 3h / CC 15min) | **Priority**: P1
- **File**: `services/backend/src/infra/reinfolib_mock.rs`

---

## P2 — SaaS化フェーズ

### 可観測性基盤（FE pino追加）
- **What**: FE側にpino構造化ログ追加。BE側はtracing + tracing-subscriber + telemetry crate実装済み
- **Effort**: XS | **Priority**: P2

### CI/CDパイプライン
- **What**: GitHub Actions で clippy + cargo test + npm test + build + deploy
- **Effort**: S | **Priority**: P2

### 認証方針の決定
- **What**: JWT vs セッションベースの方向性をPhase 1で決定。Phase 2で実装
- **Why**: Axum の tower middleware 構成と AppState に影響
- **Effort**: S（決定のみ）| **Priority**: P2

### Mapbox GL JS 切替検討
- **What**: Phase 1はMapLibre維持。Globe view/terrain 3D/Mapbox Studioが必要になったら切替
- **Why**: 3箇所の変更で低リスク。従量課金はSaaS収益でカバー
- **Effort**: S | **Priority**: P2

### 全国対応の性能検証
- **What**: まず首都圏4県（東京+神奈川+埼玉+千葉）で性能テスト、bbox P99 < 100msを基準に
- **Why**: 全国データ投入でクエリ性能が劣化する可能性。パーティショニングの必要性を検証
- **Effort**: M | **Priority**: P2

### UIUX_SPEC.md 更新
- **What**: CRT/Shadowbrokerテーマ参照をUrban Stratigraphyデザインシステムに更新
- **Why**: ステールなドキュメントは新エンジニアの混乱を招く。globals.cssとの乖離が拡大
- **Effort**: S (human 4h / CC 15min) | **Priority**: P2
- **File**: `docs/UIUX_SPEC.md`

### DESIGN.md 作成
- **What**: Urban Stratigraphyデザインシステムの公式ドキュメント（カラートークン、タイポグラフィ、コンポーネントパターン）
- **Why**: globals.cssにデザイントークンが定義済みだがドキュメント化されていない。21レイヤー拡大後、デザインレビューの基準が不明確
- **Effort**: S (human 4h / CC 15min) | **Priority**: P2
- **File**: `DESIGN.md`

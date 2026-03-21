# TODOS

> 最終更新: 2026-03-22 (PR1完了後の整理)

---

## P0 — 現フェーズで対応

### Vitest テスト基盤構築（FE）
- **What**: フロントエンドのvitest基盤 + PR1で追加した21レイヤーシステムのテスト
- **Why**: テストゼロのまま21レイヤーを運用するのは危険。コンポーネントレジストリ、レイヤー設定、ストア、ポップアップ等の回帰テスト必須
- **Effort**: M (human 1day / CC 15min) | **Priority**: P0
- **Scope**: layers.ts設定整合性、map-store toggleLayer/selectFeature、PopupCard描画、YearSliderデバウンス、useAreaData hook

### Rust Axum バックエンド移行（2週間）
- **What**: Python FastAPI (~200行) → Rust Axum に書き換え。PostGIS 移行と同時実施
- **Why**: 全国GeoJSON (数GB) の bbox フィルタリング + 投資スコア空間計算がホットパス
- **Effort**: L | **Priority**: P0
- **Plan**: `docs/plans/2026-03-19-rust-axum-migration.md`
- **Schedule**: Week 1 scaffold + PostGIS Docker、Week 2 データ投入 + API
- **CRITICAL**: SQLは必ずパラメータバインド（`$1`, `$2`）。`format!()`でSQL埋め込み禁止
- **CRITICAL**: APIエンドポイントは `/api/area-data` を維持（リネームしない）

### PostGIS seed データ作成
- **What**: 開発用最小サンプルデータ（東京駅周辺5-10行/テーブル）
- **Why**: PostGISにデータ投入される前の開発段階でFE開発者が動作確認できない（DEMO_MODE撤去後の開発体験断絶を防ぐ）
- **Effort**: XS | **Priority**: P0
- **Depends on**: Rust Axum移行（PostGISスキーマ）と同時

---

## P1 — Phase 1内で対応

### cargo test テスト基盤構築（BE: Rust移行と並行）
- **What**: `cargo test` + `#[sqlx::test]`
- **Why**: テストゼロのまま新バックエンドを投入するのは危険
- **Effort**: M | **Priority**: P1
- **最低限**: `/api/health` smoke test + scoring engine 単体テスト + `#[sqlx::test]` で /api/area-data 統合テスト

### セキュリティ強化（Rust移行と同時）
- **What**: bbox範囲制限、CORS明示設定（環境変数 `ALLOWED_ORIGINS`）、レート制限（tower-governor）、入力バリデーション
- **Why**: bbox無制限でDoS可能。CorsLayer::permissive() のまま本番投入は危険
- **Effort**: S | **Priority**: P1

### XKT025/026 代替データ経路の確定
- **What**: 液状化(XKT025)と洪水(XKT026)のreinfolib APIエンドポイントが存在しない可能性。代替ソースを確定させる
- **Why**: 現在のDEMO_MODEでは動作するが実データ移行時に破綻
- **Effort**: S | **Priority**: P1
- **対応案**: 液状化→東京都建設局SHP変換、洪水→国土数値情報A31のGeoJSON

### layers.ts の endpoint フィールド整理
- **What**: PostGIS移行後、reinfolib APIコード（XPT002等）を保持する `endpoint` フィールドがdead codeになる
- **Effort**: XS | **Priority**: P1

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

### reinfolib API モック/スタブ
- **What**: reinfolib_mock.rsでモックレスポンスを提供し、APIキー取得前の並行開発を可能にする
- **Why**: APIキー申請は2-4週かかる可能性。Phase 2の不動産取引価格レイヤー(#8)開発をブロックしない
- **Effort**: S (human 3h / CC 15min) | **Priority**: P1
- **File**: `services/backend/src/infra/reinfolib_mock.rs`

---

## P2 — SaaS化フェーズ

### 可観測性基盤
- **What**: `tracing` + `tracing-subscriber` (BE: Rust移行で同時導入) + pino (FE)
- **Effort**: S | **Priority**: P2

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

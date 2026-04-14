# UIUX_SPEC.md — UI/UX 設計仕様書

> Version: 3.0.0 | Updated: 2026-04-14
> Design Language: Terrasight — mapleads レイアウト + rakumachi 情報設計
> Framework: Next.js 16 + Mapbox GL JS + react-map-gl + Tailwind CSS v4 + shadcn/ui

---

## 1. デザイン原則

### 1.1 ビジュアルコンセプト

**mapleads レイアウト + rakumachi 情報設計** — 地図を常にフルスクリーン背景に保ちながら、白い不透明パネルをオーバーレイで重ねる mapleads スタイルのレイアウトを採用。情報設計は rakumachi のテーマ別排他レイヤーモデルに倣い、1テーマ = 1レイヤーセット + 1詳細パネル構成で投資判断に必要な情報を簡潔に提示する。

### 1.2 デザインルール

1. **地図常時可視**: 地図は常にフルスクリーン背景。どのパネルが開いても地図は可視状態を維持する
2. **テーマ排他切替**: 1テーマ = 1レイヤーセット + 1詳細パネル構成。複数テーマの同時表示は行わない
3. **パネルオーバーレイ**: パネルは不透明白背景で地図の上に overlay する (split view ではない)
4. **ビジュアルスタイル**: mapleads 踏襲 — 角丸 12-16px、ソフトシャドウ `rgba(0,0,0,0.08)`、0.3s ease トランジション
5. **状態遷移管理**: パネル間の導線は State 0-3 の状態遷移で管理する

---

## 2. デザイントークン

詳細は `docs/DESIGN.md` Section 2 参照。ソースオブトゥルース: `services/frontend/src/app/globals.css` および `src/lib/palette.ts`。

### 2.1 ベーストークン概要（抜粋）

| トークン | Light 値 | Dark 値 | 用途 |
|---------|----------|---------|------|
| `--bg-primary` | `#FFFFFF` | `#0c0c14` | パネル背景、ページ背景 |
| `--bg-secondary` | `#F9FAFB` | `#13131e` | セカンダリサーフェス |
| `--text-primary` | `#111827` | `#e4e4e7` | 本文テキスト、データ値 |
| `--text-secondary` | `#6B7280` | `#a1a1aa` | ラベル、説明文 |
| `--border-primary` | `rgba(0,0,0,0.08)` | `rgba(63,63,70,0.5)` | パネルボーダー |
| `--shadow-panel` | `rgba(0,0,0,0.08)` | `rgba(0,0,0,0.4)` | パネルドロップシャドウ |

### 2.2 アクセントカラー

| トークン | 値 | 用途 |
|---------|-----|------|
| `--accent-indigo` | `#6366F1` | プライマリアクセント、アクティブインジケーター、フォーカスリング |
| `--accent-indigo-tint` | `rgba(99,102,241,0.12)` | サイドバー・テーブル行のアクティブハイライト |
| `--hover-blue-tint` | `rgba(59,130,246,0.06)` | ホバー状態背景 |
| `--accent-danger` | `#e04030` | 高リスク表示、破壊的アクション |
| `--accent-warning` | `#ffd000` | 警告、中リスク表示 |
| `--accent-success` | `#10b981` | 成功状態 |

### 2.3 レイヤーカラートークン (`--layer-*`)

Mapbox GL paint expression は CSS カスタムプロパティを参照できないため、`--layer-*` トークンは raw hex 定数として `src/lib/palette.ts` に定義し、`globals.css` と Mapbox paint expression の両方が参照する。詳細は `docs/DESIGN.md` Section 2.3 参照。

---

## 3. レイアウト構成

### 3.1 4 State システム

AppShell は以下 4 つの離散レイアウト状態を管理する。

```
State 0: Sidebar + Map (初期状態)
┌──────┬─────────────────────────────────────┐
│ Nav  │            Mapbox Map               │
│56/200│     (ライト/ダーク/サテライト)        │
│      │                        [Style Sw.]  │
│      │                        [NavCtrl]    │
└──────┴─────────────────────────────────────┘
│ StatusBar (28px)                           │
└────────────────────────────────────────────┘

State 1: Sidebar + Left Panel + Map (地点詳細)
┌──────┬────────────┬────────────────────────┐
│ Nav  │ Left Panel │        Map             │
│      │  360px     │                        │
│      │            │            [Style Sw.] │
│      │            │            [NavCtrl]   │
└──────┴────────────┴────────────────────────┘
│ StatusBar (28px)                           │
└────────────────────────────────────────────┘

State 2: Sidebar + Table + Map (Opportunities)
┌──────┬──────────────────────────────────────┐
│ Nav  │         Mapbox Map (full)            │
│      │                                      │
│      │  ┌──────────────────────────────┐    │
│      │  │ Opportunities Table (~65% w) │    │
│      │  │ [virtualized rows]           │    │
└──────┴──┴──────────────────────────────┴────┘
│ StatusBar (28px)                             │
└──────────────────────────────────────────────┘
テーブルはサイドバーと同じフルハイト、地図の上に overlay。

State 3: Sidebar + Table + Right Drawer + Map
┌──────┬───────────────────────┬──────────────┐
│ Nav  │     Mapbox Map        │   Right      │
│      │                       │   Drawer     │
│      │  ┌──────────────┐     │   340px      │
│      │  │ Table (~65%w)│     │              │
│      │  │              │     │              │
└──────┴──┴──────────────┴─────┴──────────────┘
│ StatusBar (28px)                             │
└──────────────────────────────────────────────┘
テーブルは State 2 と同位置 (フルハイト)。Drawer は右端にフルハイトで overlay。
```

### 3.2 状態遷移ルール

- `leftPanel` と `tableOpen` は排他。両方同時に `true` にはならない
- `rightDrawer` は `tableOpen === true` の時のみ有効
- `activeTheme` 変更時は `leftPanel` を閉じる (State 1 → State 0)
- Opportunities ナビクリック時は `leftPanel` を閉じてから `tableOpen` を開く

### 3.3 主要寸法

| 要素 | 寸法 |
|-----|------|
| Sidebar (折りたたみ) | 幅 56px、全高 minus status bar |
| Sidebar (展開) | 幅 200px、全高 minus status bar |
| Left Detail Panel | 幅 360px、全高 minus status bar |
| Opportunities Table | ビューポート幅 ~65%、下部アンカー |
| Right Drawer | 幅 340px、全高 minus status bar |
| Status bar | 高さ 28px、全幅 |
| パネル角丸 | 12-16px |

### 3.4 z-index スタック

| z-index | 要素 |
|---------|------|
| 100 | Right Drawer |
| 80 | Opportunities Table |
| 60 | Left Detail Panel |
| 40 | Sidebar |
| 20 | Map controls / Legend / Style Switcher / StatusBar |
| 1 | Mapbox Map |

---

## 4. コンポーネント仕様

### 4.1 AppShell

**責務**: ルートレイアウト。全パネル・マップを包含し、4状態のレイアウトを Zustand store フラグ (`leftPanelOpen`, `tableOpen`, `rightDrawerOpen`) で制御する。

**Tailwind クラス**: `relative h-screen w-screen overflow-hidden`

構成: Sidebar + メインエリア（Mapbox Map + オーバーレイパネル群）

---

### 4.2 Sidebar (mapleads 式)

**責務**: テーマ選択ナビゲーション + Opportunities 起動。

```
┌──────────────────┐
│ [Ts] ロゴ        │
├──────────────────┤
│ ── 探す ──       │
│ 🔍 Opportunities │
│ 📊 スコア分析    │
├──────────────────┤
│ ── 見る ──       │
│ 💰 地価          │
│ 🏠 取引事例      │
│ 🌊 ハザード      │
│ 🚉 乗降客数      │
├──────────────────┤
│ ── 設定 ──       │
│ ⚙ 設定           │  ← マップスタイル切替含む
└──────────────────┘
幅: 展開 200px / 折りたたみ 56px
z-index: 40
```

**スタイル:**
- 背景: 白不透明、ソフトシャドウ `rgba(0,0,0,0.08)`、右辺に 12px 角丸
- 展開/折りたたみ: `width` トランジション 0.3s ease
- 展開時: アイコン + ラベル表示
- 折りたたみ時: アイコンのみ + Tooltip

**インタラクション:**
- アクティブ項目: `rgba(99,102,241,0.12)` (indigo tint) 背景ハイライト
- ホバー: `rgba(59,130,246,0.06)` (blue tint) 背景
- 折りたたみトグル: レール最下部に配置

---

### 4.3 Left Detail Panel (rakumachi 式)

**責務**: 地図クリック地点のテーマ連動詳細情報表示。

```
┌─────────────────────┐
│ ✕ 閉じる             │
│ 所在地テキスト        │
├─────────────────────┤
│ [地価][ハザード][駅]  │  ← テーマ連動タブ
├─────────────────────┤
│ メイン数値 + 前年比   │
│ 詳細テーブル          │
│ 年次推移グラフ        │
└─────────────────────┘
幅: 360px 固定
位置: サイドバーの右隣、z-60
```

**スタイル:**
- 背景: 白不透明
- ボーダー: 右辺にソフトシャドウ `rgba(0,0,0,0.08)`
- 角丸: `0 12px 12px 0`
- スライドイン: `translateX(-100%)` → `translateX(0)` 0.3s ease

**タブ:**
- アクティブテーマがデフォルト選択タブ
- データのある他テーマも表示 (shadcn/ui `Tabs`)

**コンテンツ (テーマ別):**

| テーマ | 表示内容 |
|--------|---------|
| 地価 | 価格、地積、用途地域、年次推移 |
| ハザード | 浸水深、土砂区域、リスクレベル |
| 取引事例 | 取引価格、面積、構造、建築年 |
| 乗降客数 | 乗降客数、前年比、年次推移 |
| スコア分析 | TLS スコア内訳、サブスコア |

---

### 4.4 Opportunities Table (mapleads CRM 式)

**責務**: Opportunity 一覧の表示・絞り込み・ソート。

```
┌──────────────────────────────────────────────┐
│ 検索 | +Filter | 件数 | ページネーション | Export | ✕ │
│ 都道府県 | 市区町村 | 金額 | Preset           │
├──────────────────────────────────────────────┤
│ 所在地 | TLS | 地価 | リスク | トレンド | 最寄駅 │
│ ─────────────────────────────────────────── │
│ (仮想化行)                                    │
└──────────────────────────────────────────────┘
位置: サイドバー右隣、下部から上方向にスライドイン
幅: ビューポート幅 ~65%
z-index: 80
```

**スタイル:**
- 背景: 白不透明
- 角丸: 上辺 12px (`12px 12px 0 0`)
- 上部にソフトシャドウ

**機能:**
- `@tanstack/react-virtual` で仮想化
- 行ホバー: `rgba(59,130,246,0.06)` 薄い青ハイライト
- 行クリック → 右 Drawer に Opportunity 詳細 (State 2 → State 3)
- アクティブ行: `rgba(99,102,241,0.12)` indigo tint

---

### 4.5 Right Drawer

**責務**: Opportunity 詳細またはテーブル中の地図地点詳細を表示。

```
┌─────────────────────┐
│ ✕ 閉じる             │
│ [Detail] [Compare]  │  ← タブ
├─────────────────────┤
│ コンテンツエリア      │
│                     │
│ (A) Opportunity 詳細 │
│   TLS スコア         │
│   サブスコアレーダー  │
│   リスク             │
│   最寄駅             │
│                     │
│ (B) 地図地点詳細     │
│   テーマ連動フォーマット│
└─────────────────────┘
幅: 340px 固定
位置: テーブル右隣、z-100
```

**スタイル:**
- 背景: 白不透明
- 角丸: `12px 0 0 12px`
- ソフトシャドウ
- スライドイン: `translateX(100%)` → `translateX(0)` 0.3s ease

**表示条件**: `tableOpen === true` の時のみ有効

**2種コンテンツ:**
- A) テーブル行クリック: TLS スコア + サブスコアレーダーチャート + リスク + 最寄駅
- B) テーブル中の地図クリック: テーマ連動フォーマット

**実装**: shadcn/ui `Sheet` (right side)

---

### 4.6 Map View

**責務**: Mapbox GL JS による地図描画とユーザーインタラクション管理。

**マップ設定:**

| プロパティ | 値 |
|-----------|-----|
| ライブラリ | Mapbox GL JS (react-map-gl 経由) |
| Token 環境変数 | `NEXT_PUBLIC_MAPBOX_TOKEN` |
| デフォルトスタイル | `mapbox://styles/mapbox/streets-v12` (Light) |
| Dark スタイル | `mapbox://styles/mapbox/dark-v11` |
| Satellite スタイル | `mapbox://styles/mapbox/satellite-streets-v12` |
| デフォルト中心 | `[139.767, 35.681]`（東京駅） |
| デフォルト zoom | `12` |
| デフォルト pitch | `0`（フラット — データ可読性優先） |
| デフォルト bearing | `0` |
| 3D buildings | オプション（デフォルト OFF、opt-in） |
| Terrain | オプション（デフォルト OFF、opt-in） |
| Move debounce | 300ms |

**SSR 対策**: `useEffect(() => setMounted(true), [])` でクライアントサイド確認後にのみ `<MapGL>` をレンダリング。

**WebGL recovery**: コンテキストロスト時にオーバーレイ表示 + 自動リカバリ待機 + 手動再読み込みボタン。

---

### 4.7 Map Style Switcher

**責務**: Light / Dark / Satellite の 3 択ベースマップ切替。

- shadcn/ui `Toggle` コンポーネント使用
- 設定セクション内に配置（Sidebar 設定メニュー）
- マップスタイル切替時: `map.setStyle()` → `style.load` イベント後にレイヤーソースと paint expression を再適用

---

### 4.8 Legend Panel (ハザードテーマ時)

**責務**: ハザードテーマ有効時に地図右下へ凡例を表示。

- 位置: 地図右下、z-20
- 折りたたみ可 (shadcn/ui `Collapsible`)
- ハザードテーマ以外では非表示

---

### 4.9 StatusBar（下部ステータスバー）

```
┌──────────────────────────────────────────────────────────┐
│  35.6812°N 139.7671°E  │  Z:12.0  │  ◌ LOADING...       │
└──────────────────────────────────────────────────────────┘
h: 28px, font-size: 10px, --font-mono
bg: --bg-primary
border-top: --border-primary
z-index: 20
```

- 座標: 現在のビューステート lat/lng
- ズームレベル: 現在のズーム値
- Loading インジケーター: データ取得中に表示（モノスペースドットアニメーション）

---

## 5. テーマシステム

### 5.1 概要

24 レイヤーの個別トグルシステムを廃止し、テーマベースの排他切替システムに置換。テーマを切り替えると前テーマのレイヤーが非表示になり（opacity 0、0.3s fade）、新テーマのレイヤーが表示される（opacity 1、0.3s fade）。

ソースオブトゥルース: `services/frontend/src/lib/themes.ts` および `docs/designs/map-visualization-spec.md`

### 5.2 利用可能テーマ

| テーマ ID | 表示名 | 地図表示形式 | 詳細パネル内容 |
|----------|--------|------------|--------------|
| `landprice` | 地価 | エリアポリゴン choropleth + 金額ラベル | 価格、地積、用途地域、年次推移 |
| `hazard` | ハザード | カラーポリゴン + 凡例 | 浸水深、土砂区域、リスクレベル |
| `transactions` | 取引事例 | クラスタ / ポイント | 取引価格、面積、構造、建築年 |
| `ridership` | 乗降客数 | 駅バブル + 人数ラベル | 乗降客数、前年比、年次推移 |
| `score` | スコア分析 | ヒートマップ / グラデーション | TLS スコア内訳、サブスコア |

各テーマの正確なレイヤーセットと paint expression の詳細は `docs/designs/map-visualization-spec.md` を参照。

### 5.3 レイヤー ID 規則

- UI レイヤー ID: `underscore_case` (例: `land_price`, `flood_risk`)
- WASM / FlatGeobuf レイヤー ID: `hyphen-case` (例: `land-price`, `flood-risk`)
- 境界を越える際は `canonicalLayerId()` (`src/lib/layers.ts`) を使用

---

## 6. インタラクション仕様

### 6.1 テーマ切替

1. ユーザーが Sidebar のテーマ項目をクリック
2. `activeTheme` が Zustand store で更新される
3. 前テーマのレイヤーが fade out (opacity 0、0.3s)
4. 新テーマのレイヤーが fade in (opacity 1、0.3s)
5. Left Detail Panel が開いていれば閉じる (State 1 → State 0)

### 6.2 地図クリック (State 0)

1. ユーザーが地図フィーチャーをクリック
2. Left Detail Panel がスライドイン (State 0 → State 1)
3. パネルにテーマ連動の地点詳細タブを表示

### 6.3 地図クリック (State 2 — テーブル表示中)

1. ユーザーが Opportunities Table 表示中に地図フィーチャーをクリック
2. Right Drawer がスライドイン (State 2 → State 3)
3. Drawer にクリック地点の詳細を表示 (テーマ連動フォーマット)

### 6.4 テーブル行クリック

1. ユーザーが Opportunities Table の行をクリック
2. Right Drawer がスライドイン (State 2 → State 3)
3. Drawer に Opportunity 詳細 (TLS スコア + レーダー + リスク + 最寄駅) を表示
4. タブ: Detail / Compare

### 6.5 Opportunities ナビクリック

1. ユーザーが Sidebar の Opportunities をクリック
2. Left Detail Panel が開いていれば先に閉じる
3. Opportunities Table が下から上にスライドイン (State 0/1 → State 2)

### 6.6 Sidebar 折りたたみ/展開

- 折りたたみトグルをクリック
- Sidebar 幅が 200px ↔ 56px でアニメーション (0.2s ease)
- 地図キャンバスはリフローしない (サイドバーは地図の上に overlay)

### 6.7 マップパン/ズーム

- moveEnd debounce: 300ms
- debounce 後に `useAreaData` の bbox を更新 → API レイヤーのデータ再取得
- bbox 最大幅制限: 0.5度 (`BBOX_MAX_DEGREES`)
- ViewState は debounce 後にのみ TanStack Query の queryKey に使用 (リクエストフラッド防止)

### 6.8 URL 状態同期

- `useMapUrlState` hook で URL ↔ 地図状態を双方向同期 (nuqs ベース)
- パラメータ: `lat`, `lng`, `z`, `pitch`, `bearing`, `theme`
- 例: `?lat=35.681&lng=139.767&z=12&theme=landprice`

---

## 7. レスポンシブ対応

デスクトップ優先設計。モバイル対応は本計画のスコープ外。

### 7.1 ブレークポイント

| 画面幅 | Sidebar | Left Panel | Table | Right Drawer |
|--------|---------|------------|-------|--------------|
| >= 1280px (desktop) | 展開/折りたたみ切替可 | 固定 360px | ~65% 幅 | 固定 340px |
| 768-1279px (tablet) | アイコンのみ (56px) | 固定 360px | ~80% 幅 | 固定 300px |
| < 768px (mobile) | アイコンのみ (56px) | 全幅スライドイン | 全幅スライドイン | TBD |

### 7.2 タッチ操作

- ピンチズーム、2本指回転対応 (Mapbox GL デフォルト)
- パネルスワイプ閉じ: 左 Panel は左スワイプ、Right Drawer は右スワイプで閉じる (TBD)

---

## 8. アクセシビリティ

- **コントラスト**: WCAG AA — 全テキストはパネル背景上で 4.5:1 以上
- **キーボードナビゲーション**: Tab でパネル間移動、Escape でパネル閉じる、Enter でアクション実行
- **ARIA ランドマーク**: Left Detail Panel と Right Drawer は `role="complementary"` + `aria-label`
- **テーブル**: `role="grid"` + `aria-rowcount` (仮想化行対応)
- **Sidebar ナビ**: 各項目に `aria-label` (テーマ名)、アクティブ状態に `aria-current="page"`
- **フォーカストラップ**: Right Drawer 展開時 (shadcn/ui Sheet の挙動)
- **Map canvas**: `aria-label="Interactive map"` + `role="application"`
- **色のみ依存禁止**: リスクレベルはアイコン + テキスト (LOW/MEDIUM/HIGH) でも表示
- **reduced-motion**: `prefers-reduced-motion` 時はスライドアニメーション無効化、opacity フェードのみ

---

## 9. ローディング / エラー / 空状態

### 9.1 ローディング

- Left Detail Panel: テーマ連動データ取得中は shadcn/ui `Skeleton` 行
- Right Drawer: Opportunity 詳細形状に合わせた `Skeleton` レイアウト
- Opportunities Table: 初回フェッチ解決まで先頭 20 行を `Skeleton` 表示
- Map layers: Mapbox ソースローディングはネイティブ処理; 追加スピナー不要
- StatusBar: アクティブフェッチ中にローディングインジケーター表示

### 9.2 エラー

- パネルフェッチエラー: インラインエラーメッセージ + リトライボタン (パネルは開いたまま)
- Map ソースエラー: shadcn/ui `Sonner` トースト通知 (右上); 地図はインタラクティブのまま
- Opportunities Table フェッチエラー: 空状態 + リトライ CTA
- エラーバウンダリ: Next.js App Router の `error.tsx` で実装

### 9.3 空状態

- ビューポート内にデータなし: 空の FeatureCollection (地図上に何も描画されない)
- テーマ未選択: Sidebar でテーマを選択するよう促すメッセージ

---

## 10. ドメインモデルとインタラクション (ward 対応含む)

> このセクションは次フェーズ実装の target state。
> 現状実装は `prefecture | municipality` のみで、`ward` 対応は未了。

### 10.1 行政階層

```ts
type AreaLevel = "prefecture" | "municipality" | "ward";
```

- `prefecture` — 47 都道府県
- `municipality` — 市町村
- `ward` — 東京 23 特別区 + 政令指定都市の行政区。`municipality` とは独立した level として扱う

### 10.2 選択状態モデル (target shape)

```ts
type SelectedArea = {
  code: string;
  level: "prefecture" | "municipality" | "ward";
  parentCode: string | null;
  prefName: string;
  cityName: string | null;   // prefecture の場合 null
  wardName: string | null;   // 該当しない場合 null
  bbox: { south: number; west: number; north: number; east: number };
};
```

- `name` フィールドは canonical には含めない。表示名は UI で `wardName ?? cityName ?? prefName` の優先順で派生する
- 理由: `name` は level ごとに意味が変わり、breadcrumb / popup / API 間で解釈が分裂する

### 10.3 行政界クリック動作 (全 level 共通)

行政界クリックは単一トランザクションとして扱う:

1. `selectedArea` を更新
2. 対応境界をハイライト
3. パンくずを更新
4. `selectedArea.code` で area stats を再取得
5. 既存 Left Detail Panel を閉じる

> 行政界クリックは地図フィーチャークリックより優先する。パネルを残すと選択スコープとインスペクト対象が不整合になりやすい。

#### 状態遷移例

- **未選択 → 都道府県**: 都道府県境界クリック → `level="prefecture"`, `prefName` 設定, `cityName`/`wardName` = `null`
- **都道府県 → 市区町村**: 選択中都道府県内で市区町村境界クリック → `level="municipality"`, `parentCode` = 都道府県 code
- **市区町村 → 行政区**: 区境界をクリック → `level="ward"`, `parentCode` = 市区町村 code

### 10.4 パンくず表示ルール

| level | 表示 |
|-------|------|
| `prefecture` | `東京都` |
| `municipality` | `東京都 / 新宿区` |
| `ward` | `神奈川県 / 横浜市 / 中区` |

- 上位要素クリックで親階層へ戻る
- 戻る操作でも Left Detail Panel は閉じた状態を維持
- breadcrumb 更新は `selectedArea` のみから決定できる (他 state に依存しない)

### 10.5 `admin_boundary` の責務分離

`admin_boundary` は 2 つの責務に分離する:

1. **Base orientation boundary** — 常時表示の基盤レイヤー。位置関係把握のためユーザーのテーマ切替には依存しない
2. **Interactive boundary settings** — 強調表示 ON/OFF、ラベル濃度、click-interaction の有効化などの設定層

### 10.6 UIState (Zustand)

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `activeTheme` | `ThemeId \| null` | 現在アクティブなテーマ |
| `mapStyle` | `'light' \| 'dark' \| 'satellite'` | 現在のベースマップスタイル |
| `leftPanelOpen` | `boolean` | Left Detail Panel の表示状態 |
| `tableOpen` | `boolean` | Opportunities Table の表示状態 |
| `rightDrawerOpen` | `boolean` | Right Drawer の表示状態 |
| `selectedOpportunityId` | `string \| null` | テーブルで選択された行 |
| `selectedMapFeature` | `GeoJSON.Feature \| null` | 地図上でクリックされたフィーチャー |

### 10.7 ViewState (react-map-gl)

`useMap` hook で管理。Zustand `viewState` → TanStack Query `queryKey` は必ず debounce (300ms) を通す。

| フィールド | 型 |
|-----------|-----|
| `longitude` | `number` |
| `latitude` | `number` |
| `zoom` | `number` |
| `pitch` | `number` |
| `bearing` | `number` |

### 10.8 Opportunity (コアデータ型)

| フィールド | 型 | 説明 |
|-----------|-----|------|
| `id` | `string` | UUID |
| `coordinates` | `[number, number]` | `[lng, lat]` (RFC 7946) |
| `address` | `string` | 人間可読アドレス |
| `ward` | `string` | 東京 23 区名 |
| `landPrice` | `number \| null` | ¥/m²、最新年度 |
| `zoningCode` | `string` | 用途地域区分コード |
| `floodRiskLevel` | `number \| null` | 0-4 順序スケール |
| `compositeScore` | `number \| null` | 0-100 投資スコア (TLS) |
| `transactionCount` | `number` | 記録された取引件数 |
| `createdAt` | `string` | ISO 8601 |

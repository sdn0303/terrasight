# UIUX_SPEC.md — UI/UX 設計仕様書

> Version: 2.0.0 | Updated: 2026-03-23
> Design Language: Urban Stratigraphy — 都市の地層を可視化するダークテーマ
> Framework: Next.js 16 + MapLibre GL JS + react-map-gl + Tailwind CSS v4 + framer-motion

---

## 1. デザイン原則

### 1.1 ビジュアルコンセプト
**Urban Stratigraphy（都市地層）** — 都市を構成する見えない層（地価・リスク・地盤・インフラ・行政区画）を、地質学の地層断面のように重ねて可視化する。暗い地盤の上にデータの層が浮かび上がるダークテーマで、投資判断に必要な情報を直感的に読み取れるようにする。

### 1.2 デザインルール
1. **情報密度優先**: 投資判断に必要な情報を一画面に集約。スクロールを最小化
2. **視覚的階層**: 重要度の高いデータ（スコア、価格）は大きく・明るく。補助データはmuted
3. **アニメーションは目的的**: framer-motionはパネルのスライドイン/アウトとローディング状態のみ。装飾的アニメーション禁止
4. **タイポグラフィの使い分け**: UIラベル・本文は`Geist Sans`。数値・座標・技術データは`Geist Mono`
5. **レイヤーカラーシステム**: 24レイヤーそれぞれに固有のCSS変数カラーを割り当て、パネルのドット・マップ描画・凡例を統一

---

## 2. デザイントークン（CSS変数）

### 2.1 基盤トークン

```css
:root {
  /* ─── Urban Stratigraphy design tokens ──────── */

  /* Backgrounds */
  --bg-primary: #0c0c14;
  --bg-secondary: #13131e;
  --bg-tertiary: #1a1a28;

  /* Text */
  --text-primary: #e4e4e7;
  --text-secondary: #a1a1aa;
  --text-muted: #52525b;
  --text-heading: #f4f4f5;

  /* Borders */
  --border-primary: rgba(63, 63, 70, 0.5);

  /* Accent Colors */
  --accent-cyan: #22d3ee;      /* Primary accent, prices, positive */
  --accent-danger: #e04030;    /* Risk, declining, alerts */
  --accent-warning: #ffd000;   /* Caution, demo badge */
  --accent-success: #10b981;   /* Safe, facilities, rising */

  /* Interactive */
  --hover-accent: rgba(34, 211, 238, 0.08);

  /* Typography */
  --font-mono: 'Geist Mono', monospace, system-ui;
  --font-sans: 'Geist Sans', system-ui, sans-serif;
}
```

### 2.2 レイヤーカラートークン

24レイヤーそれぞれに固有の `--layer-*` CSS変数を定義。パネルのインジケータードット、マップの描画色、PopupCardのヘッダーに一貫して使用される。

```css
:root {
  /* ─── Layer color tokens ─────────────────────── */
  --layer-landprice: #fbbf24;       /* 地価公示 */
  --layer-flood-history: #60a5fa;   /* 浸水履歴 */
  --layer-did: #a78bfa;             /* 人口集中地区 */
  --layer-flood: #0ea5e9;           /* 洪水浸水 */
  --layer-steep-slope: #f97316;     /* 急傾斜地 */
  --layer-fault: #ef4444;           /* 断層線 */
  --layer-volcano: #f43f5e;         /* 火山 */
  --layer-landform: #d4a574;        /* 地形分類 */
  --layer-geology: #8b7355;         /* 表層地質 */
  --layer-soil: #a0845c;            /* 土壌図 */
  --layer-schools: #34d399;         /* 学校 */
  --layer-medical: #2dd4bf;         /* 医療機関 */
  --layer-boundary: #a1a1aa;        /* 市町村境界 */
  --layer-zoning: #818cf8;          /* 用途地域 */
  --layer-station: #f472b6;         /* 鉄道駅 */
  --layer-school-dist: #4ade80;     /* 小学校区 */
  --layer-landslide: #fb923c;       /* 土砂災害 */
  --layer-park: #86efac;            /* 都市公園 */
  --layer-population: #c084fc;      /* 将来人口メッシュ */
  --layer-urban-plan: #34d399;      /* 立地適正化 */
  --layer-tsunami: #38bdf8;         /* 津波浸水 */
  --layer-liquefaction: #eab308;    /* 液状化危険度 */
  --layer-seismic: #ef4444;         /* 地震動・震源断層 */
  --layer-railway: #22d3ee;         /* 鉄道路線 */
}
```

### 2.3 shadcn/ui テーマ統合

shadcn/ui コンポーネント（Sheet, Collapsible, Skeleton 等）は `zinc dark` ベースのカスタムテーマで、Urban Stratigraphy トークンと統合:

- `--primary` → `--accent-cyan` (#22d3ee)
- `--background` → `--bg-primary` (#0c0c14)
- `--card` → `--bg-secondary` (#13131e)
- `--destructive` → `--accent-danger` (#e04030)
- `--ring` → `--accent-cyan` (#22d3ee)

---

## 3. レイアウト構成

### 3.1 全体レイアウト

```
┌─────────────────────────────────────────────────────────────────┐
│                                                                 │
│  ┌──────────┐                                                  │
│  │          │                                                  │
│  │  Layer   │         3D Map (MapLibre GL)                     │
│  │  Panel   │       (pitch: 45, bearing: 0)                    │
│  │  (Left)  │       CARTO Dark Matter basemap                  │
│  │  w:280   │       + 3D buildings + terrain DEM               │
│  │          │                                                  │
│  │  地層    │                              ┌───────────┐      │
│  │  URBAN   │                              │  Score     │      │
│  │  STRATI- │                              │  Card      │      │
│  │  GRAPHY  │                              │  (Right)   │      │
│  │          │                              │  w:320     │      │
│  └──────────┘                              └───────────┘      │
│                                                                 │
│          [PopupCard — click-inspect overlay, z-30]              │
│          [YearSlider — population mesh control]                 │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Dashboard Stats Bar (Bottom)  h:120                     │  │
│  │  [AVG PRICE] [LISTINGS] [RISK] [FACILITIES]              │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Status Bar (Bottom)  h:28                               │  │
│  │  [座標] [ズーム] [DEMO badge] [Loading indicator]        │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 z-index スタック

| z-index | 要素 |
|---------|------|
| 100 | ComparePanel（モーダルオーバーレイ） |
| 50 | Mobile LayerPanel trigger button |
| 40 | LayerPanel（左パネル、デスクトップ） |
| 30 | PopupCard / DashboardStats |
| 20 | StatusBar |
| 10 | MapLibre NavigationControl |
| 1 | MapLibre Map |

---

## 4. コンポーネント仕様

### 4.1 MapView

**責務**: 3Dダークマップの描画とユーザーインタラクション管理。

```
┌──────────────────────────────────────────────┐
│                                              │
│              3D Dark Map                     │
│           (pitch: 45, bearing: 0)            │
│           CARTO Dark Matter basemap          │
│                                              │
│     ▓▓▓ 3D buildings (fill-extrusion)       │
│     ███ Terrain DEM (elevation)             │
│                                              │
│     + 24 data layers (toggle via panel)     │
│                                              │
│                               ┌──────┐      │
│                               │ N    │      │
│                               │ ◄ ►  │      │
│                               │ + -  │      │
│                               └──────┘      │
└──────────────────────────────────────────────┘
```

**Props:**
- `children: ReactNode` — Source/Layerコンポーネント
- `onMoveEnd: () => void` — debounced bbox更新
- `onFeatureClick: (e: MapLayerMouseEvent) => void`

**初期ビューステート:**
- center: `[139.767, 35.681]`（東京駅）
- zoom: `12`
- pitch: `45`
- bearing: `0`
- style: CARTO Dark Matter (`dark-matter-gl-style`)

**3D features:**
- Terrain DEM: `elevation-tiles-prod/terrainrgb` (exaggeration: 1.5)
- 3D Buildings: `fill-extrusion` from CARTO vector tiles (`#1e1e2e`, opacity 0.7)

**SSR対策**: `mounted` state guard — `useEffect(() => setMounted(true), [])` でクライアントサイド確認後にのみ `<MapGL>` をレンダリング。ロード中は `地層 LOADING...` を表示。

**WebGL recovery**: コンテキストロスト時にオーバーレイ表示 + 自動リカバリ待機 + 手動再読み込みボタン。

---

### 4.2 LayerPanel（左パネル）

**責務**: 24レイヤーを5カテゴリに分類し、ON/OFF切替UIを提供。

```
┌─────────────────────────┐
│ 地層                     │  ← プロジェクト名
│ URBAN STRATIGRAPHY       │  ← サブタイトル（mono）
│                          │
│ ▸ 投資価値         [2]  │  ← カテゴリ（折りたたみ）
│   ● 地価公示             │     + アクティブ数バッジ
│   ○ 浸水履歴             │
│   ○ 人口集中地区         │
│   ○ 鉄道駅              │
│                          │
│ ▸ リスク評価       [0]  │
│   ○ 洪水浸水             │
│   ○ 急傾斜地             │
│   ○ 液状化危険度         │
│   ○ 地震動・震源断層     │
│   ○ 断層線              │
│   ○ 火山                │
│   ○ 土砂災害            │
│   ○ 津波浸水            │
│                          │
│ ▸ 地盤             [0]  │
│   ○ 地形分類             │
│   ○ 表層地質             │
│   ○ 土壌図              │
│                          │
│ ▸ インフラ         [0]  │
│   ○ 学校                │
│   ○ 医療機関            │
│   ○ 小学校区            │
│   ○ 都市公園            │
│   ○ 鉄道路線            │
│                          │
│ ▸ オリエンテーション [2] │
│   ● 市町村境界           │
│   ● 用途地域             │
│   ○ 将来人口メッシュ     │
│   ○ 立地適正化           │
└─────────────────────────┘
w: 280px
bg: --bg-secondary
border-right: --border-primary
```

**5カテゴリ:**

| カテゴリID | ラベル | 説明 |
|-----------|--------|------|
| `value` | HOW MUCH? / 投資価値 | 地価・浸水履歴・DID・駅 |
| `risk` | IS IT SAFE? / リスク評価 | 洪水・急傾斜・液状化・地震・断層・火山・土砂・津波 |
| `ground` | WHAT'S THE GROUND? / 地盤 | 地形・地質・土壌 |
| `infra` | WHAT'S NEARBY? / インフラ | 学校・医療・学区・公園・鉄道 |
| `orientation` | WHERE AM I? / オリエンテーション | 境界・用途地域・人口メッシュ・立地適正化 |

**各レイヤートグル:**
- ON: `var(--layer-*)` カラードット + `--text-primary` テキスト + `--hover-accent` 背景
- OFF: `--text-muted` ドット + muted テキスト + transparent 背景
- カテゴリヘッダーに折りたたみ（Collapsible）+ アクティブレイヤー数バッジ

**初期状態:**
- `landprice`: ON
- `admin_boundary`: ON
- `zoning`: ON
- 他: OFF
- カテゴリ `value` と `risk` はデフォルト展開

**レスポンシブ:**
- Desktop (>=1280px): 固定 280px 左パネル（AnimatePresence でスライドイン/アウト）
- Mobile/Tablet (<1280px): shadcn/ui Sheet (左サイド) + MenuIcon トリガーボタン

---

### 4.3 PopupCard（クリック検査ポップアップ）

**責務**: マップ上のフィーチャーをクリックした時に、レイヤー設定に基づくフィールド情報を表示。

```
┌────────────────────────┐
│  地価公示               │  ← layerNameJa（cyan）
├────────────────────────┤
│  所在地     千代田区... │
│  価格       1,200,000  │  ← suffix: 円/㎡
│  変動率     +3.2       │  ← suffix: %
└────────────────────────┘
max-w: 240px
bg: --bg-secondary
border: --border-primary
font: --font-mono, 11px
```

**データ駆動設計**: `layers.ts` の `popupFields` 配列に基づき動的にレンダリング。レイヤーごとの個別テンプレートは不要。各フィールドは `{ key, label, suffix? }` で定義。

---

### 4.4 ScoreCard（右パネル）

**責務**: クリックしたフィーチャーの詳細情報 + 投資スコア表示。

```
┌──────────────────────────────┐
│  PROPERTY INTEL              │  ← header
├──────────────────────────────┤
│                              │
│  LOCATION                    │
│  千代田区丸の内1             │
│                              │
│  ┌────────────────────────┐  │
│  │  INVESTMENT SCORE       │  │
│  │       ╭─────╮          │  │
│  │      ╱   72  ╲         │  │  ← ScoreGauge（半円SVG arc）
│  │     ╱─────────╲        │  │
│  │    0    50    100       │  │
│  │                         │  │
│  │  trend: 18/25           │  │  ← ComponentBar
│  │  risk:  22/25           │  │
│  │  access:15/25           │  │
│  │  yield: 17/25           │  │
│  └────────────────────────┘  │
│                              │
│  ┌────────────────────────┐  │
│  │  PRICE TREND            │  │
│  │  ╱‾‾‾╲   ╱‾‾‾‾‾       │  │  ← Sparkline（5年間推移）
│  │ ╱      ╲╱              │  │
│  │ 2020        2024       │  │
│  │        CAGR: +3.2%     │  │
│  └────────────────────────┘  │
│                              │
│  ...                        │
└──────────────────────────────┘
w: 320px
position: fixed right-4 top-24
bg: rgba(--bg-primary, 0.9) + backdrop-blur-md
animation: slide-in from right (framer-motion x:320→0)
```

**投資スコアゲージ (ScoreGauge):**
- 0-33: `--accent-danger`（赤）
- 34-66: `--accent-warning`（黄）
- 67-100: `--accent-success`（緑）
- 半円メーター（SVG arc）、中央に数値

**ComponentBar**: 各スコア要素（trend, risk, access, yield）のバー表示

**Sparkline**: 価格推移の折れ線グラフ（上昇: `--accent-success` / 下降: `--accent-danger`）

---

### 4.5 DashboardStats（下部バー）

**責務**: ビューポート連動のリアルタイムエリア統計。TanStack Query で bbox 変更時に `/api/stats` を呼び出し。

```
┌──────────────────────────────────────────────────────────────┐
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐│
│  │ AVG PRICE│  │ LISTINGS │  │ RISK     │  │ FACILITIES   ││
│  │ ¥850,000 │  │   45     │  │  18%     │  │  13          ││
│  │ med:720k │  │          │  │          │  │ 3 sch, 10 med││
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘│
└──────────────────────────────────────────────────────────────┘
```

**各カード (StatCard):**
- bg: `--bg-tertiary`
- ラベル: `--text-muted`, 9px, tracking-[0.15em]
- 値: `--accent-cyan`, text-lg, font-bold
- 副値: `--text-secondary`, 10px
- RISK値: 30%超で `--accent-danger`、以下で `--accent-success`

**レスポンシブ:**
- Desktop (>=1280px): h:120px, flex row, 固定下部
- Tablet (768-1279px): h:80px, 2x2 grid
- Mobile (<768px): デフォルト非表示。フローティングボタンでトグル表示

**ローディング**: shadcn/ui Skeleton パルスアニメーション

---

### 4.6 ComparePanel（比較パネル）

**責務**: 2地点のサイドバイサイド比較。recharts RadarChart で可視化。

```
┌────────────────────────────────────────────────────────┐
│  COMPARE ANALYSIS                              [×]    │
├────────────────────────────────────────────────────────┤
│                                                        │
│  ┌──────────────┐          ┌──────────────┐           │
│  │ POINT A      │          │ POINT B      │           │
│  │ 千代田区     │          │ 新宿区       │           │
│  │ Score: 72    │          │ Score: 65    │           │
│  └──────────────┘          └──────────────┘           │
│                                                        │
│           ┌──────────────────────┐                    │
│           │   Radar Chart        │                    │
│           │      地価            │                    │
│           │     ╱    ╲           │                    │
│           │  利回   安全性       │                    │
│           │     ╲    ╱           │                    │
│           │    医療  教育        │                    │
│           │                      │                    │
│           │  ── A (cyan)         │                    │
│           │  ── B (warning)      │                    │
│           └──────────────────────┘                    │
└────────────────────────────────────────────────────────┘
z-index: 100 (modal overlay)
bg: --bg-secondary + backdrop-blur(12px)
backdrop: rgba(0,0,0,0.6)
animation: scale-in (framer-motion scale:0.9→1)
```

**レーダーチャート軸:** 地価 / 安全性 / 教育 / 医療 / 利回り

**カラー:**
- Point A: `--accent-cyan`
- Point B: `--accent-warning`

---

### 4.7 StatusBar（下部ステータスバー）

```
┌──────────────────────────────────────────────────────────┐
│  35.6812°N 139.7671°E  │  Z:12.0  │  ● DEMO  │  ◌ LOADING... │
└──────────────────────────────────────────────────────────┘
h: 28px, font-size: 10px, --font-mono
bg: --bg-primary
border-top: --border-primary
z-index: 20
```

- 座標: 現在のビューステート lat/lng
- ズームレベル: 現在のズーム値
- DEMOバッジ: `--accent-warning` で表示（APIキー未設定時）
- Loadingインジケーター: `--accent-cyan` で表示（データ取得中）

---

### 4.8 YearSlider（人口メッシュ年度スライダー）

**責務**: 将来人口メッシュレイヤーの表示年度を選択。`population_mesh` レイヤーがアクティブな時のみ表示。

---

## 5. レイヤーシステム（24レイヤー）

### 5.1 レイヤー構成

全24レイヤーは `layers.ts` で一元管理。各レイヤーは `LayerConfig` 型で定義:

```typescript
interface LayerConfig {
  id: string;
  name: string;           // English name
  nameJa: string;         // Japanese display name
  category: "value" | "risk" | "ground" | "infra" | "orientation";
  defaultEnabled: boolean;
  color: string;          // CSS variable reference (e.g., "var(--layer-landprice)")
  source: "api" | "static";
  popupFields?: PopupField[];
  interactiveLayerIds?: string[];
  minZoom?: number;
}
```

### 5.2 データソース分類

**API layers** (6): `useAreaData` hook で bbox に基づきバックエンドからフェッチ。

| ID | nameJa | カテゴリ | カラー |
|----|--------|---------|--------|
| `landprice` | 地価公示 | value | `--layer-landprice` (#fbbf24) |
| `flood` | 洪水浸水 | risk | `--layer-flood` (#0ea5e9) |
| `steep_slope` | 急傾斜地 | risk | `--layer-steep-slope` (#f97316) |
| `schools` | 学校 | infra | `--layer-schools` (#34d399) |
| `medical` | 医療機関 | infra | `--layer-medical` (#2dd4bf) |
| `zoning` | 用途地域 | orientation | `--layer-zoning` (#818cf8) |

**Static layers** (18): `/geojson/` からマウント時にロード。

| ID | nameJa | カテゴリ | カラー |
|----|--------|---------|--------|
| `flood_history` | 浸水履歴 | value | `--layer-flood-history` (#60a5fa) |
| `did` | 人口集中地区 | value | `--layer-did` (#a78bfa) |
| `station` | 鉄道駅 | value | `--layer-station` (#f472b6) |
| `liquefaction` | 液状化危険度 | risk | `--layer-liquefaction` (#eab308) |
| `seismic` | 地震動・震源断層 | risk | `--layer-seismic` (#ef4444) |
| `fault` | 断層線 | risk | `--layer-fault` (#ef4444) |
| `volcano` | 火山 | risk | `--layer-volcano` (#f43f5e) |
| `landslide` | 土砂災害 | risk | `--layer-landslide` (#fb923c) |
| `tsunami` | 津波浸水 | risk | `--layer-tsunami` (#38bdf8) |
| `landform` | 地形分類 | ground | `--layer-landform` (#d4a574) |
| `geology` | 表層地質 | ground | `--layer-geology` (#8b7355) |
| `soil` | 土壌図 | ground | `--layer-soil` (#a0845c) |
| `school_district` | 小学校区 | infra | `--layer-school-dist` (#4ade80) |
| `park` | 都市公園 | infra | `--layer-park` (#86efac) |
| `railway` | 鉄道路線 | infra | `--layer-railway` (#22d3ee) |
| `admin_boundary` | 市町村境界 | orientation | `--layer-boundary` (#a1a1aa) |
| `population_mesh` | 将来人口メッシュ | orientation | `--layer-population` (#c084fc) |
| `urban_plan` | 立地適正化 | orientation | `--layer-urban-plan` (#34d399) |

### 5.3 レイヤーコンポーネントアーキテクチャ

`page.tsx` のレジストリパターンで、レイヤーIDからReactコンポーネントへのマッピングを一元管理:

- `STATIC_LAYER_COMPONENTS`: `{ visible: boolean }` を受け取る
- `API_LAYER_COMPONENTS`: `{ data: FeatureCollection, visible: boolean }` を受け取る
- `PopulationMeshLayer`: 追加で `{ selectedYear: number }` を受け取る

各レイヤーコンポーネントは個別の MapLibre paint expression を持ち、`components/map/layers/` に配置。

---

## 6. インタラクション仕様

### 6.1 マップパン/ズーム
- moveEnd debounce: 300ms (`DEBOUNCE_MS`)
- debounce後に `useAreaData` の bbox を更新 → API層レイヤーのデータ再取得
- bbox最大幅制限: 0.5度 (`BBOX_MAX_DEGREES`)

### 6.2 フィーチャークリック（click-inspect）
1. `interactiveLayerIds`（全24レイヤーから収集）に該当するフィーチャーをクリック
2. `selectedFeature` を Zustand store に保存（layerId, properties, coordinates）
3. layerId プレフィックスから `layers.ts` の `popupFields` を逆引き
4. PopupCard を画面中央に表示（config-driven、レイヤーごとのテンプレート不要）
5. マップの空白部分をクリック → `selectFeature(null)` でPopupCard閉じる

### 6.3 比較モード
1. 比較モード有効化 → クリック動作が切り替わる
2. 1つ目の地点をクリック → Point A として記録（address or lat/lng）
3. 2つ目の地点をクリック → Point B として記録 → ComparePanel 表示
4. 各 Point に対して `/api/score` を呼び出し → RadarChart 描画
5. ComparePanel の「×」またはバックドロップクリック → 比較モード終了

### 6.4 URL状態同期
- `useMapUrlState` hook で URL ↔ 地図状態を双方向同期（nuqs ベース）
- パラメータ: `lat`, `lng`, `z`, `pitch`, `bearing`, `layers` (カンマ区切り)
- 例: `?lat=35.681&lng=139.767&z=12&pitch=45&layers=landprice,zoning`

---

## 7. レスポンシブ対応

### 7.1 ブレークポイント

| 画面幅 | LayerPanel | ScoreCard | DashboardStats |
|--------|-----------|-----------|---------------|
| >= 1280px (desktop) | 左固定 280px (AnimatePresence) | 右固定 320px | 下部固定 h:120px, flex row |
| 768-1279px (tablet) | Sheet（左サイド、MenuIcon trigger） | 右固定 280px | 下部固定 h:80px, 2x2 grid |
| < 768px (mobile) | Sheet（左サイド、MenuIcon trigger） | ボトムシート | 非表示（フローティングボタンでトグル） |

### 7.2 モバイル対応方針
- デスクトップ優先
- モバイルは「見れる」レベル（フル機能は不要）
- タッチ操作: ピンチズーム、2本指回転対応（MapLibreデフォルト）
- モバイル LayerPanel: shadcn/ui Sheet でスライドイン

---

## 8. アクセシビリティ

- カラーコントラスト: WCAG AA（ダークテーマのため text-primary on bg-primary で4.5:1以上）
- キーボードナビゲーション: Tab でパネル間移動、Escape でパネル閉じる
- スクリーンリーダー: `aria-label` をレイヤートグル・ステータスバー・スコア値に付与
- `aria-pressed` でレイヤートグル状態を公開
- `aria-expanded` でカテゴリ折りたたみ状態を公開
- `role="status"` + `aria-live="polite"` で StatusBar 更新を通知
- 色だけに依存しない: リスクレベルはテキスト（LOW/MEDIUM/HIGH）でも表示

---

## 9. ローディング/エラー/空状態

### 9.1 ローディング
- StatusBar: `◌ LOADING...` テキスト（`--accent-cyan`）
- DashboardStats: shadcn/ui Skeleton パルスアニメーション
- MapView初期化: `地層 LOADING...` テキスト

### 9.2 エラー
- WebGL context lost: モーダルオーバーレイ + `⚠ 地図を再読み込み中...` + 手動再読み込みボタン
- 部分的なデータ取得失敗: 失敗したレイヤーの data は空 FeatureCollection にフォールバック

### 9.3 空状態
- ビューポート内にデータなし: 空の FeatureCollection（マップ上に何も描画されない）
- レイヤー全OFF: パネルで有効化を促す

---

## 10. ドメインモデルとインタラクション (ward 対応含む)

> このセクションは旧 `TERRASIGHT_SPEC_V1.md` から統合された次フェーズ実装の target state。
> 現状実装は `prefecture | municipality` のみで、`ward` 対応は未了。

### 10.1 行政階層

```ts
type AreaLevel = "prefecture" | "municipality" | "ward";
```

- `prefecture` — 47 都道府県
- `municipality` — 市町村
- `ward` — 東京 23 特別区 + 政令指定都市の行政区。`municipality` とは独立した level として扱う

### 10.2 選択状態モデル (target shape)

現状の `SelectedArea` を拡張する target shape:

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

- **`name` フィールドは canonical には含めない**。表示名は UI で `wardName ?? cityName ?? prefName` の優先順で派生する。
- 理由: `name` は level ごとに意味が変わり、breadcrumb / popup / API 間で解釈が分裂する。

### 10.3 行政界クリック動作 (全 level 共通)

行政界クリックは単一トランザクションとして扱う:

1. `selectedArea` を更新
2. 対応境界をハイライト
3. パンくずを更新
4. `selectedArea.code` で area stats を再取得
5. 既存 popup を閉じる

> **行政界クリックは click-inspect popup より優先する**。popup を残すと選択スコープと
> inspect 対象が不整合になりやすい。

#### 状態遷移例

- **未選択 → 都道府県**: 都道府県境界クリック → `level="prefecture"`, `prefName` 設定, `cityName`/`wardName` = `null`
- **都道府県 → 市区町村**: 選択中都道府県内で市区町村境界クリック → `level="municipality"`, `parentCode` = 都道府県 code
- **市区町村 → 行政区**: 区境界をクリック → `level="ward"`, `parentCode` = 市区町村 code

### 10.4 パンくず表示ルール

| level | 表示 |
|---|---|
| `prefecture` | `東京都` |
| `municipality` | `東京都 / 新宿区` |
| `ward` | `神奈川県 / 横浜市 / 中区` |

- 上位要素クリックで親階層へ戻る
- 戻る操作でも popup は閉じた状態を維持
- breadcrumb 更新は `selectedArea` のみから決定できる (他 state に依存しない)

### 10.5 `admin_boundary` の責務分離

`admin_boundary` は単一概念ではなく 2 つの責務に分離する:

1. **Base orientation boundary** — 常時表示の基盤レイヤー。位置関係把握のためユーザーの layer toggle には強く依存しない。表示が途切れて「どことどこに跨っているか分からない」状態を作らない
2. **Interactive boundary settings** — 強調表示 ON/OFF、ラベル濃度、click-interaction の有効化などの設定層。Base orientation を消すためのものではない

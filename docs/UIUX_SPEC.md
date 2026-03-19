# UIUX_SPEC.md — UI/UX 設計仕様書

> Version: 1.0.0 | Updated: 2026-03-20
> Design Language: Shadowbroker-inspired CRT Dark Theme
> Framework: Next.js 16 + MapLibre GL JS + react-map-gl + Tailwind CSS + framer-motion

---

## 1. デザイン原則

### 1.1 ビジュアルコンセプト
**Shadowbroker-style CRT Dark** — 軍事/諜報系のダッシュボードを彷彿とさせるダークテーマ。CRTモニターのビネット効果とスキャンラインで没入感を演出。データは「インテリジェンスレポート」のように表示する。

### 1.2 デザインルール
1. **情報密度優先**: 投資判断に必要な情報を一画面に集約。スクロールを最小化
2. **視覚的階層**: 重要度の高いデータ（スコア、価格）は大きく・明るく。補助データはmuted
3. **アニメーションは目的的**: framer-motionはパネルのスライドイン/アウトとローディング状態のみ。装飾的アニメーション禁止
4. **テキストはモノスペース**: 数値・コード・座標は`Geist Mono`。UIラベルもmonospace
5. **色は最小限**: cyan（アクセント）、red（危険/下降）、yellow（警告）、green（安全/上昇）のみ

---

## 2. デザイントークン（CSS変数）

```css
:root {
  /* Backgrounds */
  --bg-primary: #0a0a0f;
  --bg-secondary: #12121a;
  --bg-tertiary: #1a1a25;

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
  --accent-warning: #ffd000;   /* Caution, zoning */
  --accent-success: #10b981;   /* Safe, facilities, rising */

  /* Interactive */
  --hover-accent: rgba(34, 211, 238, 0.1);

  /* Typography */
  --font-mono: 'Geist Mono', monospace, system-ui;
  --font-size-label: 9px;     /* Tracking labels (PROPERTY INTEL, LOCATION) */
  --font-size-data: 10px;     /* Secondary data values */
  --font-size-value: 14px;    /* Primary data values */
  --font-size-heading: 13px;  /* Section headings */
  --letter-spacing-label: 0.15em;
}
```

---

## 3. レイアウト構成

### 3.1 全体レイアウト

```
┌─────────────────────────────────────────────────────────────────┐
│ [CRT Scanline Overlay — pointer-events: none, z-index: 300]    │
│ [CRT Vignette Overlay — pointer-events: none, z-index: 200]    │
│                                                                 │
│  ┌──────────┐                                    ┌───────────┐ │
│  │          │                                    │           │ │
│  │  Layer   │         3D Map                     │  Score    │ │
│  │  Panel   │       (MapLibre)                   │  Card     │ │
│  │  (Left)  │                                    │  (Right)  │ │
│  │  w:280   │                                    │  w:320    │ │
│  │          │                                    │           │ │
│  │          │                                    │           │ │
│  └──────────┘                                    └───────────┘ │
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Dashboard Stats Bar (Bottom)  h:120                     │  │
│  │  [平均地価] [物件数] [リスク分布] [取引件数]              │  │
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
| 300 | CRT scanline overlay |
| 200 | CRT vignette overlay |
| 100 | Compare Panel（モーダル的） |
| 50 | ScoreCard（右パネル） |
| 40 | LayerPanel（左パネル） |
| 30 | Dashboard Stats Bar |
| 20 | Status Bar |
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
│                                              │
│     ● 地価公示ポイント (cyan circles)        │
│     ███ 用途地域ポリゴン (色分けfill)        │
│     ▓▓▓ 防災リスク3D (fill-extrusion)        │
│     ◆ 学校マーカー (green)                   │
│     + 医療機関マーカー (emerald)              │
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
- `onMoveEnd: (bounds: BBox) => void`
- `onFeatureClick: (feature: GeoJSON.Feature) => void`

**初期ビューステート:**
- center: `[139.767, 35.681]`（東京駅）
- zoom: `12`
- pitch: `45`
- bearing: `0`

**SSR対策**: `mounted` state guard — `useEffect(() => setMounted(true), [])` でクライアントサイド確認後にのみ `<Map>` をレンダリング。

---

### 4.2 LayerPanel（左パネル）

**責務**: レイヤーのON/OFF切替UI。

```
┌─────────────────────────┐
│ ▸ REALESTATE             │  ← プロジェクト名
│   INTELLIGENCE           │
│                          │
│ ── PRICING ──────────── │  ← カテゴリ区切り
│ ● [ON ] 地価公示         │  ← toggle + icon + name
│ ● [OFF] 取引価格         │
│                          │
│ ── URBAN PLANNING ────── │
│ ● [ON ] 用途地域         │
│                          │
│ ── DISASTER RISK ─────── │
│ ● [OFF] 液状化リスク     │
│ ● [OFF] 洪水浸水         │
│ ● [OFF] 急傾斜地         │
│                          │
│ ── FACILITIES ────────── │
│ ● [OFF] 学校             │
│ ● [OFF] 医療機関         │
└─────────────────────────┘
w: 280px
bg: --bg-secondary
border-right: --border-primary
```

**各レイヤートグル:**
- ON: `--accent-cyan` ドット + 白テキスト
- OFF: `--text-muted` ドット + muted テキスト
- ホバー: `--hover-accent` 背景

**初期状態:**
- `landprice`: ON
- `zoning`: ON
- 他: OFF

---

### 4.3 ScoreCard（右パネル）

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
│  │  INVESTMENT SCORE       │  │  ← 投資スコアゲージ（新規）
│  │       ╭─────╮          │  │
│  │      ╱   72  ╲         │  │  ← 半円ゲージ（0-100）
│  │     ╱─────────╲        │  │
│  │    0    50    100       │  │
│  │                         │  │
│  │  trend: 18/25           │  │  ← コンポーネントバー
│  │  risk:  22/25           │  │
│  │  access:15/25           │  │
│  │  yield: 17/25           │  │
│  └────────────────────────┘  │
│                              │
│  ┌────────────────────────┐  │
│  │  PRICING                │  │
│  │  per sqm    ¥1,200,000 │  │
│  │  land price ¥1,200,000 │  │
│  └────────────────────────┘  │
│                              │
│  ┌────────────────────────┐  │
│  │  PRICE TREND ───────── │  │  ← Sparkline（新規）
│  │  ╱‾‾‾╲   ╱‾‾‾‾‾       │  │  ← 5年間の推移
│  │ ╱      ╲╱              │  │
│  │ 2020        2024       │  │
│  │        CAGR: +3.2%     │  │
│  └────────────────────────┘  │
│                              │
│  ZONING                      │
│  商業地域                    │
│                              │
│  ┌────────────────────────┐  │
│  │  DISASTER RISK          │  │
│  │  liquefaction    LOW    │  │  ← danger カラー
│  │  flood depth     0m     │  │
│  └────────────────────────┘  │
│                              │
│  FACILITIES                  │
│  [School] 千代田小学校       │
│  [Medical] 聖路加国際病院    │
│                              │
│  ▸ RAW DATA                  │  ← 折りたたみ
│    id: 1                     │
│    price_per_sqm: 1200000    │
│    ...                       │
└──────────────────────────────┘
w: 320px
position: fixed right-4 top-24
bg: rgba(10, 10, 15, 0.9) + backdrop-blur-md
animation: slide-in from right (framer-motion x:320→0)
```

**投資スコアゲージ:**
- 0-33: `--accent-danger`（赤）
- 34-66: `--accent-warning`（黄）
- 67-100: `--accent-success`（緑）
- 半円メーター（SVG arc）
- 中央に数値（大きいフォント）

**Sparkline:**
- 幅: 100%（パネル幅に合わせる）
- 高さ: 40px
- 上昇トレンド: `--accent-success`
- 下降トレンド: `--accent-danger`
- CAGR値を右下に表示

---

### 4.4 DashboardStats（下部バー）

**責務**: ビューポート連動のリアルタイムエリア統計。

```
┌──────────────────────────────────────────────────────────────┐
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────────┐│
│  │ AVG PRICE│  │ LISTINGS │  │ RISK     │  │ ZONING       ││
│  │ ¥850,000 │  │   45     │  │ ██░░░    │  │ ████░░       ││
│  │  /sqm    │  │          │  │  18%     │  │ 商業 35%     ││
│  │ med:720k │  │          │  │          │  │ 住居 45%     ││
│  └──────────┘  └──────────┘  └──────────┘  └──────────────┘│
└──────────────────────────────────────────────────────────────┘
h: 120px
bg: --bg-secondary
border-top: --border-primary
```

**各カード:**
- bg: `--bg-tertiary`
- ラベル: `--text-muted`, 9px, tracking-widest
- 値: `--accent-cyan`, 18px, font-bold
- 副値: `--text-secondary`, 10px

**更新タイミング**: マップのmoveEnd時に `/api/stats` を呼び出し、ダッシュボードを更新。ローディング中はskeletonアニメーション。

---

### 4.5 ComparePanel（比較パネル — 新規）

**責務**: 2地点のサイドバイサイド比較。

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
│           │                      │                    │
│           │      地価            │                    │
│           │     ╱    ╲           │                    │
│           │  利回   安全性       │                    │
│           │     ╲    ╱           │                    │
│           │    医療  教育        │                    │
│           │                      │                    │
│           │  ── A (cyan)         │                    │
│           │  ── B (yellow)       │                    │
│           └──────────────────────┘                    │
│                                                        │
│  ┌─────────────────────────────────────────────────┐  │
│  │  DETAIL COMPARISON                               │  │
│  │                     Point A      Point B         │  │
│  │  Avg Price/sqm      ¥1,200,000   ¥890,000       │  │
│  │  Flood Risk         LOW          MEDIUM          │  │
│  │  Schools (1km)      3            5               │  │
│  │  Medical (1km)      5            8               │  │
│  │  CAGR (5y)          +3.2%        +1.8%           │  │
│  └─────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
z-index: 100 (overlay)
bg: --bg-secondary + backdrop-blur
animation: scale-in (framer-motion)
```

**レーダーチャート軸:**
1. 地価（高い = 良い? → ユーザー設定可能にするかは要検討）
2. 安全性（防災リスクの反転）
3. 教育（学校アクセス）
4. 医療（医療機関アクセス）
5. 利回り

**カラー:**
- Point A: `--accent-cyan`
- Point B: `--accent-warning`

---

### 4.6 StatusBar（下部ステータスバー）

```
┌──────────────────────────────────────────────────────────┐
│  35.6812°N 139.7671°E  │  Z:12  │  ● DEMO  │  ◌ Loading │
└──────────────────────────────────────────────────────────┘
h: 28px, font-size: 10px, monospace
bg: --bg-primary
border-top: --border-primary
```

- 座標: マウス位置のlat/lng
- ズームレベル: 現在のズーム値
- DEMOバッジ: APIキー未設定時に黄色で表示
- Loadingインジケーター: データ取得中に表示

---

### 4.7 CRTOverlay

**責務**: CRTモニター風の視覚エフェクト。pointer-events: none。

```css
/* Vignette */
.crt-vignette {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 200;
  background: radial-gradient(circle, transparent 40%, rgba(0,0,0,0.8) 100%);
}

/* Scanlines */
.crt-scanlines {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 300;
  opacity: 0.05;
  background: linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px);
  background-size: 100% 4px;
}
```

---

## 5. マップレイヤースタイリング

### 5.1 地価公示（landprice）— circle layer

```javascript
paint: {
  "circle-radius": ["interpolate", ["linear"], ["zoom"], 10, 3, 15, 8],
  "circle-color": "#22d3ee",  // --accent-cyan
  "circle-opacity": 0.8,
  "circle-stroke-width": 1,
  "circle-stroke-color": "#0a0a0f",  // --bg-primary
}
```

### 5.2 用途地域（zoning）— fill layer

```javascript
paint: {
  "fill-color": ["match", ["get", "zone_type"],
    "第一種低層住居専用地域", "#2563eb",
    "第二種低層住居専用地域", "#3b82f6",
    "第一種中高層住居専用地域", "#60a5fa",
    "第二種中高層住居専用地域", "#93c5fd",
    "第一種住居地域", "#a78bfa",
    "第二種住居地域", "#c4b5fd",
    "準住居地域", "#e9d5ff",
    "近隣商業地域", "#fbbf24",
    "商業地域", "#f97316",
    "準工業地域", "#a3e635",
    "工業地域", "#6b7280",
    "工業専用地域", "#374151",
    "#6b7280"  // default
  ],
  "fill-opacity": 0.35,
}
```

### 5.3 防災リスク（flood, steep_slope）— fill-extrusion layer

```javascript
paint: {
  "fill-extrusion-color": ["interpolate", ["linear"], ["get", "risk_score"],
    0, "#1a6fff",     // 低リスク: 青
    0.5, "#ffd000",   // 中リスク: 黄
    1.0, "#e04030",   // 高リスク: 赤
  ],
  "fill-extrusion-height": ["*", ["get", "risk_score"], 200],
  "fill-extrusion-base": 0,
  "fill-extrusion-opacity": 0.7,
}
```

### 5.4 学校・医療機関 — circle layer

```javascript
// Schools
paint: {
  "circle-radius": 5,
  "circle-color": "#10b981",  // --accent-success
  "circle-opacity": 0.9,
}

// Medical
paint: {
  "circle-radius": 5,
  "circle-color": "#6ee7b7",  // emerald-300
  "circle-opacity": 0.9,
}
```

---

## 6. インタラクション仕様

### 6.1 マップパン/ズーム
- debounce: 300ms
- ズーム制限なし（PostGIS移行後はbbox面積制限がサーバー側で対応）
- パン中はローディングインジケーター非表示（debounce後に表示）

### 6.2 フィーチャークリック
1. クリックしたフィーチャーのプロパティを取得
2. ScoreCard をスライドイン表示（framer-motion: x:320→0, duration:0.3s）
3. `/api/score` を呼び出して投資スコアを取得
4. `/api/trend` を呼び出してSparklineデータを取得
5. 別のフィーチャーをクリック → ScoreCard の内容を更新
6. マップの空白部分をクリック → ScoreCard をスライドアウト

### 6.3 比較モード
1. 「比較」ボタン押下 → 比較モード有効化
2. 1つ目の地点をクリック → Point A として記録
3. 2つ目の地点をクリック → Point B として記録 → ComparePanel 表示
4. ComparePanel の「×」 → 比較モード終了

### 6.4 URL状態同期
- `useSearchParams` でURL ↔ 地図状態を双方向同期
- パラメータ: `lat`, `lng`, `z`, `pitch`, `bearing`, `layers` (カンマ区切り)
- 例: `?lat=35.681&lng=139.767&z=12&pitch=45&layers=landprice,zoning`
- ブラウザの戻る/進むで `popstate` イベントをリッスンし地図状態を復元

---

## 7. レスポンシブ対応

### 7.1 ブレークポイント

| 画面幅 | LayerPanel | ScoreCard | DashboardStats |
|--------|-----------|-----------|---------------|
| >= 1280px (desktop) | 左固定 280px | 右固定 320px | 下部固定 120px |
| 768-1279px (tablet) | 左折りたたみ（ハンバーガー） | 右固定 280px | 下部固定 80px |
| < 768px (mobile) | ボトムシート | ボトムシート | 非表示（タップで表示） |

### 7.2 モバイル対応方針
- Phase 1ではデスクトップ優先
- モバイルは「見れる」レベル（フル機能は不要）
- タッチ操作: ピンチズーム、2本指回転対応（MapLibreデフォルト）

---

## 8. アクセシビリティ

- カラーコントラスト: WCAG AA（ダークテーマのため text-primary on bg-primary で4.5:1以上）
- キーボードナビゲーション: Tab でパネル間移動、Escape でパネル閉じる
- スクリーンリーダー: aria-label をレイヤートグル・ステータスバー・スコア値に付与
- 色だけに依存しない: リスクレベルはテキスト（LOW/MEDIUM/HIGH）でも表示

---

## 9. ローディング/エラー/空状態

### 9.1 ローディング
- ステータスバーにスピナー + "LOADING..." テキスト
- DashboardStats: skeleton パルスアニメーション
- ScoreCard: 各セクションにinline skeleton

### 9.2 エラー
- バックエンド接続不可: 赤ボーダーで "BACKEND UNREACHABLE" 表示
- 部分的なデータ取得失敗: 失敗したレイヤーを非表示、console.warn

### 9.3 空状態
- ビューポート内にデータなし: "No data in this area. Try zooming in or panning."
- レイヤー全OFF: "Enable layers from the left panel"

# Terrasight Design System v3

> Version: 3.0.0 | Updated: 2026-04-21
> Supersedes: DESIGN.md v2 (mapleads light theme)
> Pencil Mockup: `/Users/sdn03/Documents/aiden-mem.pen` (11 screens)
> Design Doc: `docs/designs/2026-04-16-ui-redesign-v3.md`

---

## 1. Design Philosophy

全画面ダークマップ + フローティング UI。地図が常に主役であり、UI コンポーネントは地図の上に浮遊する半透明パネルとして存在する。

### 1.1 Design Principles

1. **地図が主役**: 全 UI は地図の上にフロート。地図操作を阻害しない
2. **探索体験の最大化**: ズームレベルに応じた LOD、ホバー/クリックで文脈的情報表示
3. **データの意味が即座にわかる**: 凡例常時表示、カテゴリタブで「今何を見ているか」明確
4. **パネル間の隙間**: コンポーネント間に 12px の隙間がありマップが見え続ける
5. **ポリゴン表現の統一**: 全タブのエリアオーバーレイはラジアルグラデーション楕円で統一（フラット矩形禁止）

### 1.2 Visual References

- **mapleads**: 全画面マップ + フローティングパネルのレイアウトパターン
- **楽待ち**: データバッジ（地図上に地価/人口/乗降客数をインライン表示）+ 詳細パネル構成

---

## 2. Color Tokens (Dark Map Theme)

Source of truth: `services/frontend/src/app/globals.css` `:root`

### 2.1 Panel Backgrounds

| Token | Value | Opacity | Usage |
|-------|-------|---------|-------|
| `--ts-bg-panel` | `#111111F2` | 95% | メインパネル背景 |
| `--ts-bg-panel-hover` | `#111111` | 100% | パネルホバー |
| `--ts-bg-sidebar` | `#111111E6` | 90% | サイドバー背景 |
| `--ts-bg-tab-active` | `#FF840033` | 20% | アクティブタブ背景 |
| `--ts-bg-overlay` | `#111111CC` | 80% | オーバーレイ背景 |
| `--ts-bg-table-alt` | `#FFFFFF05` | 2% | テーブル zebra stripe |

### 2.2 Accent

| Token | Value | Usage |
|-------|-------|-------|
| `--ts-accent` | `#FF8400` | プライマリオレンジ |
| `--ts-accent-hover` | `#FF9520` | ホバー状態 |

### 2.3 Text

| Token | Value | Usage |
|-------|-------|-------|
| `--ts-text-primary` | `#FFFFFF` | 見出し、主要テキスト |
| `--ts-text-secondary` | `#D1D5DB` | ラベル、説明文 |
| `--ts-text-muted` | `#9CA3AF` | 補足テキスト |
| `--ts-text-dim` | `#6B7280` | 非アクティブ項目 |

### 2.4 Score Colors

TLS スコアおよび各種評価値の色分け。

| Token | Value | Range | Usage |
|-------|-------|-------|-------|
| `--ts-score-excellent` | `#10B981` | 80-100 | 優良 |
| `--ts-score-good` | `#3B82F6` | 60-80 | 良好 |
| `--ts-score-average` | `#FBBF24` | 40-60 | 平均 |
| `--ts-score-caution` | `#F97316` | 20-40 | 注意 |
| `--ts-score-danger` | `#EF4444` | 0-20 | 危険 |

### 2.5 Borders

| Token | Value | Usage |
|-------|-------|-------|
| `--ts-border-subtle` | `#FFFFFF1A` | 10% white、パネル境界 |
| `--ts-border-divider` | `#FFFFFF0A` | 4% white、セクション区切り |

---

## 3. Typography

### 3.1 Font Stack

| Role | Font Family | Fallback |
|------|-------------|----------|
| UI Chrome | **Sora** | system-ui, sans-serif |
| Data Values | **IBM Plex Mono** | monospace |

### 3.2 Type Scale

| Context | Font | Size | Weight | Tracking |
|---------|------|------|--------|----------|
| Large Metric (TLS score) | Sora | 40px | 300 (Light) | -1px |
| Panel Title | Sora | 20px | 500 (Medium) | 0 |
| Section Title | Sora | 16px | 500 (Medium) | 0 |
| Label | Sora | 11px | 500 (Medium) | +0.5px |
| Body | Sora | 13px | 400 (Regular) | 0 |
| Data Value | IBM Plex Mono | 13-16px | 500 (Medium) | 0 |
| Badge | IBM Plex Mono | 11px | 500 (Medium) | 0 |
| Map Label | IBM Plex Mono | 10-11px | 500 (Medium) | 0 |

---

## 4. Spacing & Layout Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `--ts-gap-panel` | 12px | パネル間の隙間（マップ可視確保） |
| `--ts-sidebar-width` | 56px | サイドバー幅 |
| `--ts-tab-height` | 44px | タブバー高さ |
| `--ts-panel-radius` | 12px | パネル角丸 |
| `--ts-badge-radius` | 6px | バッジ角丸 |
| `--ts-panel-padding` | 16px | パネル内余白 |

---

## 5. Component Specifications

### 5.1 Floating Sidebar

アイコンナビゲーションレール。左端に常時表示。

| Property | Value |
|----------|-------|
| Width | 56px |
| Height | viewport - 24px |
| Position | `left: 12px, top: 12px` |
| Background | `--ts-bg-sidebar` (90% opacity) |
| Corner radius | 12px |
| z-index | 10 |

**Icon Stack** (44x44 hit area):

| Icon | Label | Action |
|------|-------|--------|
| `T` (logo) | Terrasight | Home |
| `compass` | 探索 | Map exploration (default) |
| `bar-chart` | スコア分析 | TLS score view |
| `receipt` | 取引 | Transaction list |
| `shield-alert` | ハザード | Hazard overlay |
| `layers` | 地盤 | Ground layers |
| `map-pin` | インフラ | Infrastructure toggle |
| `users` | 人口 | Demographics |
| `settings` | 設定 | Theme/Language |

**States**:
- Active: `fill: --ts-bg-tab-active`, icon `fill: --ts-accent`
- Inactive: transparent bg, icon `fill: --ts-text-muted`

### 5.2 Category Tab Bar

水平タブバー。10 カテゴリで排他的レイヤー切替。

| Property | Value |
|----------|-------|
| Position | `left: 84px, top: 12px` |
| Width | auto (fit content), max `viewport - 84 - 220px` |
| Height | 44px |
| Background | `--ts-bg-sidebar` |
| Overflow | horizontal scroll with fade |

**Tabs**:

| Tab | Icon | Map Content |
|-----|------|-------------|
| 総合 | `globe` | TLS ヒートマップ + 全バッジ |
| 地価 | `map` | 地価ポリゴン + 価格バッジ |
| 取引事例 | `file-text` | 取引集計ポリゴン + テーブル |
| 人口・世帯 | `users` | 人口メッシュ + DID + バッジ |
| 空室率 | `home` | 空室率ポリゴン + %バッジ |
| 乗降客数 | `circle-dot` | 駅円(比例) + 路線カラー + ラベル |
| 利回り | `percent` | 利回りポリゴン + 駅別バッジ |
| ハザード | `alert-triangle` | 洪水/液状化/断層/急傾斜 + 凡例 |
| 地盤 | `mountain` | 地形/地質/土壌ポリゴン |
| 用途地域 | `grid-3x3` | 用途別ポリゴン + 容積率ラベル |

### 5.3 Detail Panel (楽待ち型)

エリアバッジまたは駅ドットクリック時に表示される詳細パネル。

| Property | Value |
|----------|-------|
| Position | `left: 84px, top: 68px` |
| Width | 340px |
| Height | viewport - 80px |
| Background | `--ts-bg-panel` |
| Corner radius | 12px |
| z-index | 20 |
| Animation | slide-in from left, 200ms ease-out |

**Content Sections** (top to bottom):
1. Hero image (160px, aerial photo + gradient overlay)
2. Area name + subtitle
3. TLS Score (large metric 40px + 前年比 badge)
4. 6-axis breakdown: 地価トレンド / リスク / 利便性 / 生活環境 / 人口動態 / 地盤安全性
5. Divider
6. 公示地価 (metric + change% + 建蔽率/容積率/用途地域)
7. Divider
8. 人口・世帯数・空き家率 (3 columns)
9. Divider
10. 将来予測 (2030/2040/2050 rows)

### 5.4 Transaction Table

取引事例タブ選択時に表示されるフローティングテーブル。

| Property | Value |
|----------|-------|
| Position | `left: 84px, top: 68px` |
| Width | viewport - 84 - 220px |
| Height | viewport - 80 - 12px (bottom gap) |
| Background | `--ts-bg-panel` |
| z-index | 15 |

**Header**: title + count badge + filter dropdowns (種類/期間)
**Columns**: No. / 所在地 / 種別 / 取引価格(万) / 坪単価 / 面積(m2) / 最寄駅 / 徒歩 / 築年 / 構造 / 間取り
**Row height**: 40px, zebra striping with `--ts-bg-table-alt`

### 5.5 Floating Legend

タブに連動して凡例が切り替わるパネル。

| Property | Value |
|----------|-------|
| Position | `right: 12px, bottom: 12px` |
| Width | 188-208px |
| Background | `--ts-bg-panel` |
| Corner radius | 12px |
| z-index | 10 |
| Collapsible | chevron で折りたたみ可 |

### 5.6 Floating Search Bar

| Property | Value |
|----------|-------|
| Position | `right: 12px, top: 12px` |
| Background | `--ts-bg-panel` |
| Corner radius | 12px |
| z-index | 10 |

### 5.7 Map Controls

| Property | Value |
|----------|-------|
| Position | `right: 12px, center vertical` |
| z-index | 10 |
| Controls | zoom in/out, compass/bearing reset |

### 5.8 Map Badges (楽待ち型)

地図上に直接表示されるデータラベル。

| Property | Value |
|----------|-------|
| Corner radius | 6px |
| Height | 28px |
| Padding | 0 10px |
| Font | IBM Plex Mono, 11px, weight 500 |
| Position | 市区町村ポリゴンの centroid |
| Zoom behavior | zoom 8-12 で表示、zoom 13+ で非表示 |

バッジ背景色はスコア/値に応じて `--ts-score-*` トークンから選択。

---

## 6. Map Layer Visual Specifications

### 6.1 ポリゴン/ヒートマップ デザイン規約

全タブのマップ上エリア表現はラジアルグラデーション楕円で統一する。

**グラデーションストップ規約**:

| Stop | Position | Opacity (hex) | Role |
|------|----------|---------------|------|
| Center | 0.0 | 33-55% (`55`-`88`) | エリアの核心、最も濃い |
| Mid | 0.5 | 13-25% (`22`-`40`) | 緩やかな遷移 |
| Edge | 1.0 | 0% (`00`) | 完全透明、マップに溶ける |

**MapLibre GL 実装マッピング**:

| Pencil モック表現 | MapLibre 実装 |
|------------------|---------------|
| ラジアルグラデーション楕円 | `fill` with `fill-opacity` interpolation by data value |
| 楕円の重なり | ポリゴン境界の `fill-opacity` を `interpolate` でエッジに向かって下げる |
| 液状化ヒートマップ | `heatmap` レイヤータイプ (MapLibre native) |
| 色の段階 | `step` or `interpolate` expression on data property |

### 6.2 常時表示レイヤー (最前面)

| Layer ID | Source | Geometry | Style | z-order |
|----------|--------|----------|-------|---------|
| `admin-boundary-line` | admin_boundaries | MultiPolygon | stroke: white 40% opacity, 1px | 最前面 |
| `did-boundary` | FGB did | MultiPolygon | stroke: dashed purple 25%, fill: none | 最前面-1 |

### 6.3 タブ別カラーパレット

| Tab | Center Color | Range | Meaning |
|-----|-------------|-------|---------|
| 総合 | `#10B981` / `#3B82F6` / `#FBBF24` | スコア帯切替 | TLS 高=緑、中=青、低=黄 |
| 地価 | `#7C3AED` → `#DC2626` → `#F97316` → `#FBBF24` → `#22C55E` | 高→低 | 高額=紫、安=緑 |
| 空室率 | `#22C55E` → `#3B82F6` → `#FBBF24` → `#EF4444` | 低→高 | 健全=緑、危険=赤 |
| 利回り | `#22C55E` → `#3B82F6` → `#FBBF24` | 高→低 | 高利回り=緑、低=黄 |
| ハザード (洪水) | `#3B82F6` → `#7C3AED` | depth_rank | 浅=青、深=紫 |
| ハザード (液状化) | `#EF4444` → `#FBBF24` | pl_rank | heatmap レイヤー |
| 地盤 | `#92400E` / `#D97706` / `#60A5FA` / `#4ADE80` / `#A3E635` | 地形分類 | 台地=茶、低地=青、丘陵=緑 |
| 用途地域 | `#22C55E` / `#EF4444` / `#F97316` / `#3B82F6` / `#A855F7` | 用途種別 | 住居=緑、商業=赤、工業=青紫 |
| 人口 | `#8B5CF6` | 密度 | 紫メッシュ (DID 境界は点線) |

### 6.4 タブ別レイヤー定義

#### 総合タブ

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `tls-heatmap-fill` | WASM 計算メッシュ | fill | 5 色グラデーション, opacity 40% | 8 |
| `station-dot` | stations API | circle | orange, radius 9px | 11 |
| `station-label` | stations API | symbol | 駅名テキスト | 12 |
| `railway-line` | FGB railways | line | 薄灰, 1px | 8 |

#### 地価タブ

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `landprice-polygon` | land_prices → Polygon 変換 | fill | 6 段階色分け (紫/赤/橙/黄/緑/青) | 10 |
| `landprice-badge` | land_prices 集計 | symbol | エリアバッジ「新宿区 85.2万↑」 | 8 |
| `landprice-point` | land_prices API | circle | 個別ポイント + 価格ラベル | 13 |
| `appraisal-point` | land_appraisals API | circle | 菱形マーカー, 3 価格ポップアップ | 14 |

#### 取引事例タブ

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `tx-aggregation-fill` | mv_transaction_summary | fill | 件数/平均坪単価で塗り分け | 8 |
| `tx-aggregation-badge` | 同上 | symbol | 件数バッジ | 8 |
| `tx-individual-point` | transaction_prices API | circle | 物件種別で色分け | 13 |

#### 人口・世帯タブ

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `population-mesh-fill` | FGB population_mesh | fill | 人口密度で塗り分け + 年次スライダー | 12 |
| `population-badge` | population_municipality API | symbol | 「中野区 34.5万人↑ 20.8万世帯↑」 | 8 |
| `did-fill` | FGB did | fill | 薄紫塗り (このタブでのみ fill 表示) | 8 |

#### 空室率タブ

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `vacancy-fill` | vacancy_rates + admin_boundaries | fill | 空き家率% で 4 段階塗り分け | 8 |
| `vacancy-badge` | 同上 | symbol | 「13.7%↑」バッジ | 8 |

#### 乗降客数タブ

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `station-circle-scaled` | stations API | circle | passenger_count に比例した radius (5-30px) | 10 |
| `station-passenger-label` | stations API | symbol | 「新宿 9.2万人 +6.4%」 | 12 |
| `railway-colored-line` | FGB railways | line | 路線テーマカラー (operator+line_name) | 8 |

#### 利回りタブ

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `yield-fill` | mv_transaction_summary 算出 | fill | 利回り帯で 3 段階塗り分け | 8 |
| `yield-station-badge` | 同上 | symbol | 「高円寺 5.6% / AP 5.3% 区分 4.3%」 | 10 |

#### ハザードタブ

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `flood-fill` | flood_risk API | fill | depth_rank で 3 段階青系 | 10 |
| `flood-history-fill` | FGB flood_history | fill | 半透明塗り + 年次ラベル | 12 |
| `liquefaction-heatmap` | FGB liquefaction | heatmap | pl_rank を weight に (Point → heatmap) | 10 |
| `liquefaction-circle` | FGB liquefaction | circle | 個別点 (高 zoom のみ) | 14 |
| `steep-slope-fill` | steep_slope API | fill | 橙系塗り | 11 |
| `seismic-line` | FGB seismic | line | 確率で色分け | 8 |
| `fault-line` | FGB fault | line | 赤実線 + 名称ラベル | 9 |
| `volcano-point` | FGB volcano | circle | 三角マーカー + ランク | 8 |

#### 地盤タブ

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `landform-fill` | FGB landform | fill | 台地=茶, 低地=青, 丘陵=緑, 山地=黄緑 | 10 |
| `geology-fill` | FGB geology | fill | 地質区分別 | 12 |
| `soil-fill` | FGB soil | fill | 土壌カテゴリ別 | 12 |

#### 用途地域タブ

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `zoning-fill` | zoning API | fill | 8 種別色分け | 10 |
| `zoning-ratio-label` | zoning API | symbol | 「60%/200%」建蔽率/容積率 | 14 |

#### インフラ (サイドバー ON/OFF)

| Layer ID | Source | Type | Style | minZoom |
|----------|--------|------|-------|---------|
| `schools-circle` | schools API | circle | 青ピン, school_type でアイコン | 13 |
| `medical-circle` | medical_facilities API | circle | 赤十字, beds に比例したサイズ | 13 |

---

## 7. Railway Theme Colors

路線テーマカラーは `data/catalog/railway_colors.json` で管理。フロントエンドで `operator_name + "/" + line_name` をキーとしてルックアップ。マッチしない場合はデフォルト灰色 (`#6B7280`)。

**主要路線 (抜粋)**:

| Key | Color | Line |
|-----|-------|------|
| `JR東日本/山手線` | `#9ACD32` | 黄緑 |
| `JR東日本/中央線快速` | `#F15A22` | 橙 |
| `東京メトロ/銀座線` | `#FF9500` | オレンジ |
| `東京メトロ/丸ノ内線` | `#F62E36` | 赤 |
| `東京メトロ/日比谷線` | `#B5B5AC` | グレー |
| `東京メトロ/東西線` | `#009BBF` | 水色 |
| `東京メトロ/千代田線` | `#00A650` | 緑 |
| `東京メトロ/有楽町線` | `#C1A470` | ゴールド |
| `東京メトロ/半蔵門線` | `#8B76D0` | 紫 |
| `東京メトロ/南北線` | `#00ADA9` | ティール |
| `東京メトロ/副都心線` | `#9C5E31` | 茶 |
| `都営/浅草線` | `#E85298` | ピンク |
| `都営/三田線` | `#0079C2` | 青 |
| `都営/新宿線` | `#6CBB5A` | 黄緑 |
| `都営/大江戸線` | `#B6007A` | マゼンタ |

---

## 8. Map Configuration

| Property | Value |
|----------|-------|
| Library | Mapbox GL JS (via react-map-gl/mapbox) |
| Token env var | `NEXT_PUBLIC_MAPBOX_TOKEN` |
| Default style | `mapbox://styles/mapbox/dark-v11` |
| Satellite style | `mapbox://styles/mapbox/satellite-streets-v12` |
| Default center | `[139.767, 35.681]` (Tokyo) |
| Default zoom | 12 |
| Default pitch | 0 (flat, data readability priority) |
| Move debounce | 300ms |

---

## 9. z-index Stack

| z-index | Element |
|---------|---------|
| 20 | Detail Panel (click trigger) |
| 15 | Transaction Table |
| 10 | Sidebar, Tab Bar, Search, Legend, Map Controls |
| 0 | MapLibre GL Map |

# UIUX_SPEC.md — Terrasight UI/UX 設計仕様書 v3

> Version: 4.0.0 | Updated: 2026-04-21
> Supersedes: UIUX_SPEC.md v3 (mapleads light + 4-state system)
> Design Language: 全画面ダークマップ + フローティング UI + 10 カテゴリタブ
> Framework: Next.js 16 + Mapbox GL JS + react-map-gl/mapbox + Tailwind CSS v4 + shadcn/ui
> Design System: `docs/DESIGN.md` v3.0.0
> Pencil Mockup: `/Users/sdn03/Documents/aiden-mem.pen` (11 screens)

---

## 1. Layout Architecture

### 1.1 全画面マップ + フローティング UI

全 UI コンポーネントは全画面 MapLibre GL マップの上にフローティングで配置される。
パネル間には 12px の隙間を確保し、マップが常に見え続ける設計。

```
+-------------------------------------------------------------+
| [全画面 MapLibre GL マップ - z-index: 0]                       |
|                                                              |
| +--+  +----------------------------------------------+ [Q]  |
| |S |  | カテゴリタブバー (z:10)                          |      |
| |I |  | 総合|地価|取引|人口|空室率|乗降客|利回り|         |      |
| |D |  | ハザード|地盤|用途地域                           |      |
| |E |  +----------------------------------------------+      |
| |B |                                                         |
| |A |                      +------------+                     |
| |R |  [マップバッジ群]      | 凡例パネル  |                     |
| |  |  新宿区 85.2万        | (z:10)     |                     |
| |z |  港区 95.1万          +------------+                     |
| |: |                                        [+][-][C]        |
| |10|  +-----------+                                          |
| |  |  | 詳細パネル  | <- click (z:20)                         |
| |  |  | 楽待ち型    |                                         |
| +--+  +-----------+                                          |
|                                                              |
| +--------------------------------------------------------+  |
| | テーブルパネル (z:15) <- 取引事例タブ時               ^v |  |
| +--------------------------------------------------------+  |
| ~~~ マップが見える隙間 (12px) ~~~                             |
+-------------------------------------------------------------+
```

### 1.2 Component Hierarchy

```
<MapCanvas>                     <- 全画面, position: fixed
  <MapLibreMap />               <- z-0
  <AdminBoundaryLayer />        <- 常時最前面レイヤー
  <DIDOverlayLayer />           <- 常時うっすら表示
  <ActiveTabLayers />           <- タブ切替で表示内容変更
  <FloatingSidebar />           <- z-10, 左端
  <FloatingTabBar />            <- z-10, 上部中央
  <FloatingSearchBar />         <- z-10, 右上
  <FloatingLegend />            <- z-10, 右下
  <MapControls />               <- z-10, 右端
  <DetailPanel />               <- z-20, クリック時表示
  <TransactionTable />          <- z-15, 取引タブ時表示
</MapCanvas>
```

---

## 2. Frontend Architecture

### 2.1 Directory Structure

```
services/frontend/src/
+-- app/
|   +-- page.tsx                    <- 単一ページ (全画面マップ)
+-- features/
|   +-- map/
|   |   +-- MapCanvas.tsx           <- 全画面マップコンテナ
|   |   +-- layers/
|   |   |   +-- AdminBoundaryLayer.tsx  <- 常時表示
|   |   |   +-- DIDOverlayLayer.tsx     <- 常時表示
|   |   |   +-- TLSHeatmapLayer.tsx
|   |   |   +-- LandPriceLayer.tsx
|   |   |   +-- TransactionLayer.tsx
|   |   |   +-- PopulationMeshLayer.tsx
|   |   |   +-- VacancyLayer.tsx        <- NEW
|   |   |   +-- StationLayer.tsx        <- REWRITE (乗降客数比例)
|   |   |   +-- RailwayColorLayer.tsx   <- NEW (路線テーマカラー)
|   |   |   +-- YieldLayer.tsx          <- NEW
|   |   |   +-- HazardLayers.tsx        <- REWRITE (液状化 heatmap)
|   |   |   +-- GroundLayers.tsx
|   |   |   +-- ZoningLayer.tsx
|   |   |   +-- InfraLayers.tsx         <- REWRITE (beds 比例)
|   |   +-- badges/
|   |   |   +-- MapBadge.tsx            <- NEW: 楽待ち型バッジ
|   |   +-- controls/
|   |       +-- MapControls.tsx
|   |       +-- ZoomLevelIndicator.tsx
|   +-- sidebar/
|   |   +-- FloatingSidebar.tsx         <- NEW
|   +-- tabs/
|   |   +-- FloatingTabBar.tsx          <- NEW
|   |   +-- tab-configs.ts             <- タブ定義
|   +-- detail/
|   |   +-- DetailPanel.tsx             <- NEW (楽待ち型)
|   +-- transactions/
|   |   +-- TransactionTable.tsx        <- NEW (フローティングテーブル)
|   +-- legend/
|   |   +-- FloatingLegend.tsx          <- NEW
|   +-- search/
|       +-- FloatingSearchBar.tsx       <- NEW
+-- hooks/
|   +-- useTransactions.ts              <- NEW
|   +-- useAppraisals.ts               <- NEW
|   +-- usePopulation.ts               <- NEW
|   +-- useVacancy.ts                  <- NEW
|   +-- useActiveTab.ts                <- NEW (タブ状態管理)
+-- stores/
|   +-- uiStore.ts                     <- REWRITE (タブ/パネル状態)
+-- lib/
    +-- api/schemas/
    |   +-- population.ts              <- NEW
    |   +-- vacancy.ts                 <- NEW
    +-- railway-colors.ts              <- NEW
```

### 2.2 削除対象 (v2 からの破壊的変更)

| v2 Component | v3 Replacement |
|-------------|----------------|
| 固定幅サイドバー (200/56px) | FloatingSidebar (56px icon rail) |
| Opportunities ページ | DetailPanel + TransactionTable に統合 |
| スコア分析ページ | DetailPanel の TLS セクションに統合 |
| 設定ページ | サイドバー settings アイコンからドロワー |
| Right Drawer (340px) | 廃止 (DetailPanel に統合) |
| Opportunities Table (~65% w) | TransactionTable (フローティング) |
| White light theme panels | Dark semi-transparent panels |
| 5-theme system | 10-tab system |

---

## 3. State Management

### 3.1 UIState (Zustand)

```typescript
interface UIState {
  // Tab
  activeTab: TabId;
  setActiveTab: (tab: TabId) => void;

  // Detail Panel
  selectedArea: AreaSelection | null;
  setSelectedArea: (area: AreaSelection | null) => void;

  // Sidebar
  sidebarExpanded: boolean;
  toggleSidebar: () => void;

  // Infrastructure toggles
  showSchools: boolean;
  showMedical: boolean;
  toggleSchools: () => void;
  toggleMedical: () => void;

  // Legend
  legendCollapsed: boolean;
  toggleLegend: () => void;
}

type TabId =
  | 'overview'      // 総合
  | 'land-price'    // 地価
  | 'transactions'  // 取引事例
  | 'population'    // 人口・世帯
  | 'vacancy'       // 空室率
  | 'stations'      // 乗降客数
  | 'yield'         // 利回り
  | 'hazard'        // ハザード
  | 'ground'        // 地盤
  | 'zoning';       // 用途地域
```

### 3.2 状態遷移ルール

| 操作 | Before | After |
|------|--------|-------|
| タブ切替 | 任意 | DetailPanel 閉じる, 新タブのレイヤー表示 |
| バッジ/ポイントクリック | タブ表示中 | DetailPanel 開く |
| DetailPanel 閉じる | DetailPanel 表示中 | タブ表示のみ |
| 取引事例タブ選択 | 任意 | TransactionTable 表示 |
| 他タブへ切替 | 取引事例タブ | TransactionTable 閉じる |

### 3.3 URL 状態同期

`nuqs` ベースで URL と双方向同期。

| Parameter | Type | Description |
|-----------|------|-------------|
| `lat` | float | 緯度 |
| `lng` | float | 経度 |
| `z` | float | ズームレベル |
| `tab` | string | アクティブタブ ID |
| `area` | string | 選択エリアコード |

例: `?lat=35.681&lng=139.767&z=12&tab=land-price`

---

## 4. Interaction Specifications

### 4.1 タブ切替

1. ユーザーが TabBar のタブをクリック
2. `activeTab` が Zustand store で更新
3. 前タブのレイヤーが fade out (opacity 0, 300ms)
4. 新タブのレイヤーが fade in (opacity 1, 300ms)
5. DetailPanel が開いていれば閉じる
6. 凡例パネルが新タブに対応する凡例に切替

### 4.2 マップバッジクリック

1. ユーザーが市区町村バッジをクリック
2. `selectedArea` を更新
3. DetailPanel がスライドイン (left, 200ms ease-out)
4. 対応する行政区境界をハイライト

### 4.3 駅ドットクリック (乗降客数タブ)

1. ユーザーが駅円をクリック
2. DetailPanel に駅の乗降客数詳細を表示
3. 路線情報、前年比、周辺地価情報を併記

### 4.4 マップパン/ズーム

1. moveEnd debounce: 300ms
2. debounce 後に bbox 更新 → API レイヤーのデータ再取得
3. bbox 最大幅制限: 0.5 度 (`BBOX_MAX_DEGREES`)
4. ViewState は debounce 後にのみ TanStack Query の queryKey に使用

### 4.5 LOD (Level of Detail) 制御

| Zoom | 表示内容 |
|------|---------|
| 8-10 | 都道府県/市区町村ポリゴン + バッジ |
| 10-12 | 市区町村ポリゴン + バッジ + 鉄道路線 |
| 12-13 | バッジ非表示、個別ポイント出現開始 |
| 13+ | 個別ポイント + ラベル |

### 4.6 テーブルインタラクション (取引事例タブ)

- TransactionTable 表示時もマップは操作可能
- テーブル行ホバー: 対応マップポイントをハイライト
- テーブル行クリック: DetailPanel に取引詳細を表示
- フィルター: 種類 (マンション/戸建/土地) + 期間
- ソート: 全列ソート可能
- テーブル下部に 12px の隙間を確保しマップ可視

---

## 5. Data Flow

### 5.1 API Data Sources per Tab

| Tab | Primary API | Secondary API | FGB (WASM) |
|-----|-------------|---------------|------------|
| 総合 | `/api/v1/stats`, `/api/v1/score` | `/api/v1/area-data` | admin-boundary, railway |
| 地価 | `/api/v1/land-prices/aggregation` | `/api/v1/land-prices`, `/api/v1/appraisals` | - |
| 取引事例 | `/api/v1/transactions/aggregation` | `/api/v1/transactions` | - |
| 人口・世帯 | `/api/v1/population` | `/api/v1/population/geo` | population_mesh, did |
| 空室率 | `/api/v1/vacancy` | `/api/v1/vacancy/geo` | - |
| 乗降客数 | `/api/v1/area-data?layers=stations` | - | railway |
| 利回り | `/api/v1/transactions/summary` | - | - |
| ハザード | `/api/v1/area-data?layers=flood,steep_slope` | - | liquefaction, seismic, fault, volcano, flood-history |
| 地盤 | - | - | landform, geology, soil |
| 用途地域 | `/api/v1/area-data?layers=zoning` | - | - |

### 5.2 TanStack Query Hooks

| Hook | API | queryKey |
|------|-----|----------|
| `useLandPriceAggregation` | land-prices/aggregation | `['land-price-agg', bbox, prefCode]` |
| `useTransactionAggregation` | transactions/aggregation | `['tx-agg', bbox, prefCode]` |
| `useTransactions` | transactions | `['transactions', cityCode, yearFrom]` |
| `useAppraisals` | appraisals | `['appraisals', prefCode, cityCode]` |
| `usePopulation` | population | `['population', prefCode]` |
| `useVacancy` | vacancy | `['vacancy', prefCode]` |
| `useAreaData` | area-data | `['area-data', bbox, layers, zoom]` |
| `useStats` | stats | `['stats', bbox]` |
| `useScore` | score | `['score', lat, lng, preset]` |

全 hook は debounced bbox を queryKey に使用し、リクエストフラッド防止。

---

## 6. Accessibility

- **コントラスト**: WCAG AA — 全テキストはダークパネル背景上で 4.5:1 以上
- **キーボードナビゲーション**: Tab でパネル間移動、Escape でパネル閉じる
- **ARIA ランドマーク**: DetailPanel = `role="complementary"` + `aria-label`
- **テーブル**: `role="grid"` + `aria-rowcount`
- **Sidebar**: 各項目に `aria-label`、アクティブ状態に `aria-current="page"`
- **Map canvas**: `aria-label="Interactive map"` + `role="application"`
- **色のみ依存禁止**: リスクレベルはアイコン + テキストでも表示
- **reduced-motion**: `prefers-reduced-motion` 時はスライドアニメ無効、opacity フェードのみ

---

## 7. Loading / Error States

### 7.1 Loading

| Component | Loading State |
|-----------|--------------|
| DetailPanel | Skeleton rows (テーマ連動データ取得中) |
| TransactionTable | 先頭 20 行を Skeleton 表示 |
| Map layers | Mapbox source loading はネイティブ処理 |
| Map badges | Skeleton badge (灰色背景 + shimmer) |

### 7.2 Error

| Component | Error State |
|-----------|------------|
| DetailPanel | インラインエラー + リトライボタン (パネル開いたまま) |
| TransactionTable | 空状態 + リトライ CTA |
| Map source error | Sonner トースト通知 (右上); 地図はインタラクティブのまま |
| Error boundary | Next.js App Router `error.tsx` |

### 7.3 Empty State

| Condition | Display |
|-----------|---------|
| bbox 内にデータなし | 空の FeatureCollection (何も描画されない) |
| タブ未選択 | 総合タブがデフォルト表示 |
| 検索結果なし | 「該当するデータがありません」メッセージ |

---

## 8. Performance Requirements

| Metric | Target |
|--------|--------|
| タブ切替レイヤー遷移 | < 300ms |
| API レスポンス (bbox query) | < 500ms (p95) |
| Map render (10 layers) | 60fps |
| TransactionTable (1000 rows) | < 100ms initial render |
| WASM query (5 layers) | < 16ms (p95) |
| Badge rendering (50 badges) | < 16ms |

### 8.1 Performance Rules (from AGENTS.md)

- Profile before optimizing (DevTools Performance tab first)
- Zustand `viewState` → TanStack Query `queryKey` は必ず debounced state を通す
- WASM は O(n log n)+ のみ。O(n) ループは JS の方が FFI オーバーヘッドで速い
- Reduce requests before optimizing computation

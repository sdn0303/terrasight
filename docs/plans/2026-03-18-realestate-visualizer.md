# 不動産投資データビジュアライザー 実装計画

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 国土交通省 不動産情報ライブラリAPIのデータを、Shadowbrokerライクなダークテーマの3Dインタラクティブマップ上に可視化するプロトタイプを構築する。

**Architecture:** Next.js (TypeScript) フロントエンド + FastAPI (Python) バックエンドの2サービス構成。FastAPIが国交省APIのプロキシ兼キャッシュ層として動作し、GeoJSON形式でフロントへ配信。フロントエンドはreact-map-gl + MapLibre GLでダークテーマ3Dマップを描画する。Shadowbrokerのフォークではなく新規構築（UIスタイルのみ参考）。

**Tech Stack:**
- Frontend: Next.js 16 + TypeScript + react-map-gl + maplibre-gl + Tailwind CSS + framer-motion + lucide-react
- Backend: FastAPI + httpx + SQLite (aiosqlite) + uvicorn
- Map: CARTO Dark Matter basemap + MapLibre GL JS
- Data: 国交省 不動産情報ライブラリ REST API (タイルベース GeoJSON/PBF)

---

## 判断メモ

### Shadowbroker フォーク vs 新規構築 → **新規構築を選択**
- Shadowbrokerは2,400行のMaplibreViewer.tsx、航空機/船舶/衛星等20+レイヤー、AIS WebSocket等、不動産とは無関係な複雑さが大量
- 削除するコードの方が多く、依存関係の整理だけで数日かかる
- UIパターン（左パネルトグル、ダークテーマCSS変数、react-map-gl宣言的パターン）は参考にしつつ新規構築する方が速い

### API設計: タイルベースAPI直接利用 vs バックエンドプロキシ → **バックエンドプロキシ**
- 国交省APIはCORS未対応 → ブラウザ直接呼び出し不可
- APIキーをフロントエンドに露出させない
- SQLiteキャッシュでAPI呼び出しを最小化（レート制限対策）
- タイル座標変換ロジックをバックエンド側に集約

### 初期対象エリア → **東京都心（千代田・中央・港・新宿・渋谷区）**
- データ密度が高く可視化の効果が出やすい
- 都道府県コード: 13、市区町村コード: 13101-13113

---

## Task 1: プロジェクト初期化 — FastAPI バックエンド

**Files:**
- Create: `backend/main.py`
- Create: `backend/requirements.txt`
- Create: `backend/.env.example`
- Create: `backend/lib/__init__.py`
- Create: `backend/lib/tile_math.py`
- Create: `backend/lib/cache.py`

**Step 1: requirements.txt を作成**

```
fastapi==0.115.12
uvicorn[standard]==0.34.2
httpx==0.28.1
aiosqlite==0.21.0
python-dotenv==1.1.0
```

**Step 2: タイル座標変換ユーティリティを作成**

`backend/lib/tile_math.py`:
```python
import math

def latlng_to_tile(lat: float, lng: float, zoom: int = 15) -> tuple[int, int]:
    """緯度経度をXYZタイル座標に変換（zoom=15）"""
    n = 2 ** zoom
    x = int(n * (lng + 180) / 360)
    lat_rad = math.radians(lat)
    y = int(n * (1 - math.log(math.tan(lat_rad) + 1 / math.cos(lat_rad)) / math.pi) / 2)
    return x, y

def get_surrounding_tiles(lat: float, lng: float, zoom: int = 15) -> list[tuple[int, int]]:
    """中心タイル + 周囲8タイル（計9タイル）を返す"""
    cx, cy = latlng_to_tile(lat, lng, zoom)
    tiles = []
    for dx in [-1, 0, 1]:
        for dy in [-1, 0, 1]:
            tiles.append((cx + dx, cy + dy))
    return tiles

def bbox_to_tiles(south: float, west: float, north: float, east: float, zoom: int = 15) -> list[tuple[int, int]]:
    """バウンディングボックスをカバーするタイル一覧を返す（最大100タイル制限）"""
    x1, y1 = latlng_to_tile(north, west, zoom)  # top-left
    x2, y2 = latlng_to_tile(south, east, zoom)  # bottom-right
    tiles = []
    for x in range(x1, x2 + 1):
        for y in range(y1, y2 + 1):
            tiles.append((x, y))
            if len(tiles) >= 100:
                return tiles
    return tiles
```

**Step 3: SQLite キャッシュモジュールを作成**

`backend/lib/cache.py`:
```python
import aiosqlite
import json
import time
from pathlib import Path

DB_PATH = Path(__file__).parent.parent / "data" / "cache.db"

async def init_cache():
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    async with aiosqlite.connect(DB_PATH) as db:
        await db.execute("""
            CREATE TABLE IF NOT EXISTS api_cache (
                cache_key TEXT PRIMARY KEY,
                response_json TEXT NOT NULL,
                cached_at REAL NOT NULL
            )
        """)
        await db.commit()

async def get_cached(key: str, ttl_seconds: int = 86400) -> dict | None:
    async with aiosqlite.connect(DB_PATH) as db:
        cursor = await db.execute(
            "SELECT response_json, cached_at FROM api_cache WHERE cache_key = ?", (key,)
        )
        row = await cursor.fetchone()
        if row and (time.time() - row[1]) < ttl_seconds:
            return json.loads(row[0])
    return None

async def set_cached(key: str, data: dict):
    async with aiosqlite.connect(DB_PATH) as db:
        await db.execute(
            "INSERT OR REPLACE INTO api_cache (cache_key, response_json, cached_at) VALUES (?, ?, ?)",
            (key, json.dumps(data, ensure_ascii=False), time.time())
        )
        await db.commit()
```

**Step 4: FastAPI メインアプリを作成**

`backend/main.py`:
```python
import os
import logging
from contextlib import asynccontextmanager
from dotenv import load_dotenv
from fastapi import FastAPI, Query, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
import httpx

from lib.cache import init_cache, get_cached, set_cached
from lib.tile_math import latlng_to_tile, get_surrounding_tiles, bbox_to_tiles

load_dotenv()
logging.basicConfig(level=os.getenv("LOG_LEVEL", "INFO"))
logger = logging.getLogger(__name__)

REINFOLIB_BASE = "https://www.reinfolib.mlit.go.jp/ex-api/external"
API_KEY = os.getenv("REINFOLIB_API_KEY", "")
CACHE_TTL = int(os.getenv("CACHE_TTL_SECONDS", "86400"))

@asynccontextmanager
async def lifespan(app: FastAPI):
    await init_cache()
    yield

app = FastAPI(title="不動産投資データビジュアライザー API", lifespan=lifespan)
app.add_middleware(GZipMiddleware, minimum_size=500)
app.add_middleware(
    CORSMiddleware,
    allow_origins=["http://localhost:3000", "http://127.0.0.1:3000"],
    allow_methods=["*"],
    allow_headers=["*"],
)

async def fetch_reinfolib(endpoint: str, params: dict) -> dict:
    """国交省APIを呼び出し、キャッシュを利用"""
    cache_key = f"{endpoint}:{sorted(params.items())}"
    cached = await get_cached(cache_key, CACHE_TTL)
    if cached is not None:
        return cached
    if not API_KEY:
        raise HTTPException(status_code=500, detail="REINFOLIB_API_KEY not configured")
    async with httpx.AsyncClient(timeout=30) as client:
        resp = await client.get(
            f"{REINFOLIB_BASE}/{endpoint}",
            params=params,
            headers={"Ocp-Apim-Subscription-Key": API_KEY},
        )
        resp.raise_for_status()
        data = resp.json()
    await set_cached(cache_key, data)
    return data

async def fetch_tile_geojson(endpoint: str, tiles: list[tuple[int, int]], extra_params: dict | None = None) -> dict:
    """複数タイルのGeoJSONを取得してマージ"""
    all_features = []
    seen_coords = set()
    for x, y in tiles:
        params = {"response_format": "geojson", "z": 15, "x": x, "y": y}
        if extra_params:
            params.update(extra_params)
        try:
            data = await fetch_reinfolib(endpoint, params)
            for f in data.get("features", []):
                coord_key = str(f.get("geometry", {}).get("coordinates"))
                if coord_key not in seen_coords:
                    seen_coords.add(coord_key)
                    all_features.append(f)
        except Exception as e:
            logger.warning(f"Tile {x},{y} for {endpoint} failed: {e}")
    return {"type": "FeatureCollection", "features": all_features}

@app.get("/api/health")
async def health():
    return {"status": "ok", "api_key_set": bool(API_KEY)}

# --- 取引価格（非タイルAPI） ---
@app.get("/api/transactions")
async def get_transactions(
    city: str = Query("13101", description="市区町村コード"),
    year: str = Query("2024", description="取引年"),
    price_type: str = Query("02", description="01=土地のみ, 02=土地+建物"),
):
    data = await fetch_reinfolib("XIT001", {
        "city": city,
        "year": year,
        "priceClassification": price_type,
    })
    return data

# --- 地価公示ポイント（タイルAPI） ---
@app.get("/api/landprice")
async def get_landprice(
    lat: float = Query(35.681, description="中心緯度"),
    lng: float = Query(139.767, description="中心経度"),
):
    tiles = get_surrounding_tiles(lat, lng)
    return await fetch_tile_geojson("XPT002", tiles)

# --- 用途地域（タイルAPI） ---
@app.get("/api/zoning")
async def get_zoning(
    lat: float = Query(35.681, description="中心緯度"),
    lng: float = Query(139.767, description="中心経度"),
):
    tiles = get_surrounding_tiles(lat, lng)
    return await fetch_tile_geojson("XKT002", tiles)

# --- 液状化リスク（タイルAPI） ---
@app.get("/api/liquefaction")
async def get_liquefaction(
    lat: float = Query(35.681, description="中心緯度"),
    lng: float = Query(139.767, description="中心経度"),
):
    tiles = get_surrounding_tiles(lat, lng)
    return await fetch_tile_geojson("XKT025", tiles)

# --- 急傾斜地崩壊危険区域（タイルAPI） ---
@app.get("/api/steep-slope")
async def get_steep_slope(
    lat: float = Query(35.681, description="中心緯度"),
    lng: float = Query(139.767, description="中心経度"),
):
    tiles = get_surrounding_tiles(lat, lng)
    return await fetch_tile_geojson("XKT022", tiles)

# --- 洪水浸水（タイルAPI） ---
@app.get("/api/flood")
async def get_flood(
    lat: float = Query(35.681, description="中心緯度"),
    lng: float = Query(139.767, description="中心経度"),
):
    tiles = get_surrounding_tiles(lat, lng)
    return await fetch_tile_geojson("XKT026", tiles)

# --- 学校・施設（タイルAPI） ---
@app.get("/api/schools")
async def get_schools(
    lat: float = Query(35.681, description="中心緯度"),
    lng: float = Query(139.767, description="中心経度"),
):
    tiles = get_surrounding_tiles(lat, lng)
    return await fetch_tile_geojson("XKT006", tiles)

@app.get("/api/medical")
async def get_medical(
    lat: float = Query(35.681, description="中心緯度"),
    lng: float = Query(139.767, description="中心経度"),
):
    tiles = get_surrounding_tiles(lat, lng)
    return await fetch_tile_geojson("XKT010", tiles)

# --- バウンディングボックスで一括取得（マップ移動時） ---
@app.get("/api/area-data")
async def get_area_data(
    south: float = Query(...), west: float = Query(...),
    north: float = Query(...), east: float = Query(...),
    layers: str = Query("landprice,zoning", description="カンマ区切りのレイヤー名"),
):
    tiles = bbox_to_tiles(south, west, north, east)
    layer_list = [l.strip() for l in layers.split(",")]
    endpoint_map = {
        "landprice": "XPT002",
        "zoning": "XKT002",
        "liquefaction": "XKT025",
        "flood": "XKT026",
        "schools": "XKT006",
        "medical": "XKT010",
        "steep_slope": "XKT022",
        "disaster_area": "XKT016",
    }
    result = {}
    for layer in layer_list:
        ep = endpoint_map.get(layer)
        if ep:
            result[layer] = await fetch_tile_geojson(ep, tiles)
    return result

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=True)
```

**Step 5: .env.example を作成**

```
REINFOLIB_API_KEY=your_key_here
CACHE_TTL_SECONDS=86400
LOG_LEVEL=INFO
```

**Step 6: テスト — バックエンド起動確認**

```bash
cd backend
pip install -r requirements.txt
python main.py
# → http://localhost:8000/docs で Swagger UI 確認
# → http://localhost:8000/api/health で {"status": "ok", "api_key_set": false} 確認
```

**Step 7: コミット**

```bash
git add backend/
git commit -m "feat: FastAPI backend with MLIT API proxy and SQLite cache"
```

---

## Task 2: プロジェクト初期化 — Next.js フロントエンド

**Files:**
- Create: `frontend/` (Next.js プロジェクト)
- Create: `frontend/src/lib/api.ts`
- Create: `frontend/src/app/globals.css` (ダークテーマ変数)
- Create: `frontend/src/app/layout.tsx`
- Create: `frontend/src/app/page.tsx` (最小構成)

**Step 1: Next.js プロジェクトを初期化**

```bash
cd frontend
npx create-next-app@latest . --typescript --tailwind --app --src-dir --no-import-alias
npm install maplibre-gl react-map-gl framer-motion lucide-react
```

**Step 2: API クライアントを作成**

`frontend/src/lib/api.ts`:
```typescript
export const API_BASE = process.env.NEXT_PUBLIC_API_URL || "http://localhost:8000";

export async function fetchAPI<T>(path: string, params?: Record<string, string>): Promise<T> {
  const url = new URL(`${API_BASE}${path}`);
  if (params) {
    Object.entries(params).forEach(([k, v]) => url.searchParams.set(k, v));
  }
  const res = await fetch(url.toString());
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  return res.json();
}
```

**Step 3: ダークテーマ CSS 変数を設定**

`frontend/src/app/globals.css` に Shadowbroker 風のCSS変数を追加:
```css
@import "tailwindcss";

:root {
  --bg-primary: #0a0a0f;
  --bg-secondary: #12121a;
  --bg-tertiary: #1a1a25;
  --text-primary: #e4e4e7;
  --text-secondary: #a1a1aa;
  --text-muted: #52525b;
  --text-heading: #f4f4f5;
  --border-primary: rgba(63, 63, 70, 0.5);
  --accent-cyan: #22d3ee;
  --accent-danger: #e04030;
  --accent-warning: #ffd000;
  --hover-accent: rgba(34, 211, 238, 0.1);
}

body {
  background: var(--bg-primary);
  color: var(--text-primary);
  font-family: 'Geist Mono', monospace, system-ui;
}

.styled-scrollbar::-webkit-scrollbar { width: 4px; }
.styled-scrollbar::-webkit-scrollbar-track { background: transparent; }
.styled-scrollbar::-webkit-scrollbar-thumb { background: rgba(34, 211, 238, 0.2); border-radius: 2px; }
```

**Step 4: 最小の page.tsx を作成（マップ無し、接続確認用）**

```typescript
"use client";
import { useEffect, useState } from "react";
import { API_BASE } from "@/lib/api";

export default function Dashboard() {
  const [health, setHealth] = useState<{ status: string; api_key_set: boolean } | null>(null);

  useEffect(() => {
    fetch(`${API_BASE}/api/health`)
      .then(r => r.json())
      .then(setHealth)
      .catch(() => setHealth(null));
  }, []);

  return (
    <main className="fixed inset-0 flex items-center justify-center">
      <div className="bg-[var(--bg-secondary)] border border-[var(--border-primary)] rounded-xl p-8 text-center">
        <h1 className="text-2xl font-bold tracking-[0.2em] text-[var(--text-heading)] mb-4">
          不動産投資 <span className="text-[var(--accent-cyan)]">VISUALIZER</span>
        </h1>
        <p className="text-[10px] text-[var(--text-muted)] font-mono tracking-widest mb-4">
          MLIT GEOSPATIAL DATA PLATFORM
        </p>
        <div className={`text-sm font-mono ${health?.status === 'ok' ? 'text-green-400' : 'text-red-400'}`}>
          Backend: {health ? `${health.status} (API Key: ${health.api_key_set ? 'SET' : 'NOT SET'})` : 'OFFLINE'}
        </div>
      </div>
    </main>
  );
}
```

**Step 5: フロントエンド起動確認**

```bash
cd frontend && npm run dev
# → localhost:3000 でダークテーマ画面 + Backend status 表示を確認
```

**Step 6: コミット**

```bash
git add frontend/
git commit -m "feat: Next.js frontend with dark theme and API health check"
```

---

## Task 3: MapLibre ダークマップ基盤

**Files:**
- Create: `frontend/src/components/MapView.tsx`
- Modify: `frontend/src/app/page.tsx`

**Step 1: MapView コンポーネントを作成**

`frontend/src/components/MapView.tsx`:
```typescript
"use client";
import React, { useCallback, useRef, useState } from "react";
import Map, { MapRef, ViewState, NavigationControl } from "react-map-gl/maplibre";
import "maplibre-gl/dist/maplibre-gl.css";

const DARK_STYLE = "https://basemaps.cartocdn.com/gl/dark-matter-gl-style/style.json";

const INITIAL_VIEW: ViewState = {
  longitude: 139.767,
  latitude: 35.681,
  zoom: 12,
  bearing: 0,
  pitch: 45,
  padding: { top: 0, bottom: 0, left: 0, right: 0 },
};

interface MapViewProps {
  children?: React.ReactNode;
  onMoveEnd?: (bounds: { south: number; west: number; north: number; east: number }) => void;
}

export default function MapView({ children, onMoveEnd }: MapViewProps) {
  const mapRef = useRef<MapRef>(null);
  const [viewState, setViewState] = useState<ViewState>(INITIAL_VIEW);

  const handleMoveEnd = useCallback(() => {
    const map = mapRef.current?.getMap();
    if (!map || !onMoveEnd) return;
    const b = map.getBounds();
    onMoveEnd({
      south: b.getSouth(),
      west: b.getWest(),
      north: b.getNorth(),
      east: b.getEast(),
    });
  }, [onMoveEnd]);

  return (
    <Map
      ref={mapRef}
      {...viewState}
      onMove={(evt) => setViewState(evt.viewState)}
      onMoveEnd={handleMoveEnd}
      mapStyle={DARK_STYLE}
      style={{ width: "100%", height: "100%" }}
      maxPitch={60}
      antialias
    >
      <NavigationControl position="bottom-right" showCompass showZoom />
      {children}
    </Map>
  );
}
```

**Step 2: page.tsx にマップを組み込む**

page.tsx を更新し、dynamic import で MapView を読み込む（SSR回避）。

**Step 3: コミット**

```bash
git commit -m "feat: MapLibre dark map with CARTO basemap and 3D pitch"
```

---

## Task 4: 左パネル — レイヤートグルUI

**Files:**
- Create: `frontend/src/components/LayerPanel.tsx`
- Create: `frontend/src/lib/layers.ts` (レイヤー定義)
- Modify: `frontend/src/app/page.tsx`

**Step 1: レイヤー定義を作成**

`frontend/src/lib/layers.ts`:
```typescript
import { MapPin, Building2, ShieldAlert, School, Hospital, Waves, Mountain, TrendingUp } from "lucide-react";

export interface LayerDef {
  id: string;
  name: string;
  nameJa: string;
  source: string;
  icon: React.ComponentType<{ size?: number; strokeWidth?: number }>;
  defaultOn: boolean;
  category: "price" | "zoning" | "disaster" | "facility";
}

export const LAYERS: LayerDef[] = [
  { id: "landprice", name: "Land Prices", nameJa: "地価公示", source: "MLIT XPT002", icon: TrendingUp, defaultOn: true, category: "price" },
  { id: "transactions", name: "Transactions", nameJa: "取引価格", source: "MLIT XPT001", icon: MapPin, defaultOn: true, category: "price" },
  { id: "zoning", name: "Use Zones", nameJa: "用途地域", source: "MLIT XKT002", icon: Building2, defaultOn: false, category: "zoning" },
  { id: "liquefaction", name: "Liquefaction Risk", nameJa: "液状化リスク", source: "MLIT XKT025", icon: Waves, defaultOn: false, category: "disaster" },
  { id: "flood", name: "Flood Risk", nameJa: "洪水浸水", source: "MLIT XKT026", icon: Waves, defaultOn: false, category: "disaster" },
  { id: "steep_slope", name: "Steep Slope Risk", nameJa: "急傾斜地", source: "MLIT XKT022", icon: Mountain, defaultOn: false, category: "disaster" },
  { id: "schools", name: "Schools", nameJa: "学校", source: "MLIT XKT006", icon: School, defaultOn: false, category: "facility" },
  { id: "medical", name: "Medical", nameJa: "医療機関", source: "MLIT XKT010", icon: Hospital, defaultOn: false, category: "facility" },
];

export type ActiveLayers = Record<string, boolean>;

export function getDefaultActiveLayers(): ActiveLayers {
  return Object.fromEntries(LAYERS.map(l => [l.id, l.defaultOn]));
}
```

**Step 2: LayerPanel コンポーネントを作成**

Shadowbrokerの WorldviewLeftPanel パターンを参考に:
- ヘッダー（プロジェクト名 + 分類ラベル）
- カテゴリ別グループ（価格/都市計画/防災/施設）
- ON/OFF トグル + アイコン + レイヤー名

**Step 3: page.tsx で activeLayers state を管理し LayerPanel に渡す**

**Step 4: コミット**

```bash
git commit -m "feat: layer toggle panel with Shadowbroker-style dark UI"
```

---

## Task 5: 地価公示ポイント レイヤー（最初の地図レイヤー）

**Files:**
- Create: `frontend/src/hooks/useMapData.ts`
- Modify: `frontend/src/components/MapView.tsx`
- Modify: `frontend/src/app/page.tsx`

**Step 1: データフェッチ hooks を作成**

`frontend/src/hooks/useMapData.ts`:
```typescript
import { useState, useCallback } from "react";
import { API_BASE } from "@/lib/api";
import type { ActiveLayers } from "@/lib/layers";

interface Bounds {
  south: number; west: number; north: number; east: number;
}

export function useMapData(activeLayers: ActiveLayers) {
  const [layerData, setLayerData] = useState<Record<string, GeoJSON.FeatureCollection>>({});
  const [loading, setLoading] = useState(false);

  const fetchForBounds = useCallback(async (bounds: Bounds) => {
    const activeLayerIds = Object.entries(activeLayers)
      .filter(([, v]) => v)
      .map(([k]) => k)
      .filter(id => id !== "transactions"); // transactions は非タイルAPI

    if (activeLayerIds.length === 0) return;
    setLoading(true);
    try {
      const params = new URLSearchParams({
        south: bounds.south.toString(),
        west: bounds.west.toString(),
        north: bounds.north.toString(),
        east: bounds.east.toString(),
        layers: activeLayerIds.join(","),
      });
      const res = await fetch(`${API_BASE}/api/area-data?${params}`);
      if (res.ok) {
        const data = await res.json();
        setLayerData(prev => ({ ...prev, ...data }));
      }
    } catch (err) {
      console.error("Failed to fetch area data:", err);
    }
    setLoading(false);
  }, [activeLayers]);

  return { layerData, loading, fetchForBounds };
}
```

**Step 2: MapView に地価公示 circle レイヤーを追加**

react-map-gl の `<Source>` + `<Layer>` パターンで:
```tsx
{layerData.landprice && activeLayers.landprice && (
  <Source id="landprice" type="geojson" data={layerData.landprice}>
    <Layer
      id="landprice-circles"
      type="circle"
      paint={{
        "circle-radius": ["interpolate", ["linear"], ["zoom"], 10, 3, 15, 8],
        "circle-color": "#22d3ee",
        "circle-opacity": 0.8,
        "circle-stroke-width": 1,
        "circle-stroke-color": "#0a0a0f",
      }}
    />
  </Source>
)}
```

**Step 3: ホバーでポップアップ表示（地価値）**

**Step 4: コミット**

```bash
git commit -m "feat: land price circle layer with hover popup"
```

---

## Task 6: 用途地域ポリゴン レイヤー

**Files:**
- Modify: `frontend/src/components/MapView.tsx`

**Step 1: 用途地域の色マッピング定義**

```typescript
// 用途地域コード → 色マッピング
const ZONE_COLORS: Record<string, string> = {
  "第一種低層住居専用地域": "#2563eb",
  "第二種低層住居専用地域": "#3b82f6",
  "第一種中高層住居専用地域": "#60a5fa",
  "第二種中高層住居専用地域": "#93c5fd",
  "第一種住居地域": "#a78bfa",
  "第二種住居地域": "#c4b5fd",
  "準住居地域": "#e9d5ff",
  "近隣商業地域": "#fbbf24",
  "商業地域": "#f97316",
  "準工業地域": "#a3e635",
  "工業地域": "#6b7280",
  "工業専用地域": "#374151",
};
```

**Step 2: fill レイヤー（opacity 0.35）を追加**

**Step 3: コミット**

```bash
git commit -m "feat: zoning polygon layer with color-coded use types"
```

---

## Task 7: 防災リスク 3D fill-extrusion レイヤー ← 3Dの核心

**Files:**
- Modify: `frontend/src/components/MapView.tsx`
- Modify: `backend/main.py` (リスクスコア計算追加)

**Step 1: バックエンドでリスクスコアを計算して付与**

液状化・洪水・急傾斜地データをマージし、0-1のリスクスコアを各ポリゴンに付与。

**Step 2: fill-extrusion レイヤーを追加**

```tsx
<Layer
  id="disaster-risk-3d"
  type="fill-extrusion"
  paint={{
    "fill-extrusion-color": [
      "interpolate", ["linear"], ["get", "risk_score"],
      0, "#1a6fff",
      0.5, "#ffd000",
      1.0, "#e04030",
    ],
    "fill-extrusion-height": ["*", ["get", "risk_score"], 200],
    "fill-extrusion-base": 0,
    "fill-extrusion-opacity": 0.7,
  }}
/>
```

**Step 3: コミット**

```bash
git commit -m "feat: 3D disaster risk extrusion layer"
```

---

## Task 8: 施設マーカー レイヤー（学校・医療機関）

**Files:**
- Modify: `frontend/src/components/MapView.tsx`

**Step 1: 学校を symbol レイヤーで表示**

**Step 2: 医療機関を別色で表示**

**Step 3: コミット**

```bash
git commit -m "feat: school and medical facility marker layers"
```

---

## Task 9: 右パネル — ホバースコアカード

**Files:**
- Create: `frontend/src/components/ScoreCard.tsx`
- Modify: `frontend/src/app/page.tsx`

**Step 1: ScoreCard コンポーネントを作成**

マップ上のポイントをホバー/クリックした際に右パネルに表示:
- 価格情報（㎡単価、区平均比、前年比）
- 都市計画情報（用途地域、容積率）
- 防災リスクスコア（0-1、色付きバー）
- 周辺施設（学校区、最寄り医療機関）

**Step 2: page.tsx でホバー状態を管理し ScoreCard に渡す**

**Step 3: コミット**

```bash
git commit -m "feat: property scorecard panel with hover details"
```

---

## Task 10: 取引価格ヒートマップ レイヤー

**Files:**
- Modify: `frontend/src/components/MapView.tsx`
- Modify: `backend/main.py` (取引価格→GeoJSON変換)

**Step 1: バックエンドで取引価格データをジオコーディングしてGeoJSON化**

XIT001はフラットJSON（座標なし）→ 住所フィールドから概算座標を付与する処理を追加。
市区町村コード+地区名から区の中心座標をマッピングする簡易テーブルを使用。

**Step 2: heatmap レイヤーを追加**

```tsx
<Layer
  id="transactions-heatmap"
  type="heatmap"
  paint={{
    "heatmap-weight": ["interpolate", ["linear"], ["get", "price_per_sqm"], 0, 0, 2000000, 1],
    "heatmap-intensity": ["interpolate", ["linear"], ["zoom"], 8, 1, 15, 3],
    "heatmap-color": [
      "interpolate", ["linear"], ["heatmap-density"],
      0, "rgba(0,0,0,0)",
      0.2, "#1a6fff",
      0.4, "#22d3ee",
      0.6, "#ffd000",
      0.8, "#f97316",
      1, "#e04030",
    ],
    "heatmap-radius": ["interpolate", ["linear"], ["zoom"], 8, 15, 15, 30],
    "heatmap-opacity": 0.6,
  }}
/>
```

**Step 3: コミット**

```bash
git commit -m "feat: transaction price heatmap layer"
```

---

## Task 11: 仕上げ — CRTエフェクト + ステータスバー

**Files:**
- Modify: `frontend/src/app/page.tsx`
- Modify: `frontend/src/app/globals.css`

**Step 1: Shadowbroker風のCRTビネット + スキャンラインオーバーレイ**

```tsx
{/* CRT Vignette */}
<div className="absolute inset-0 pointer-events-none z-[2]"
  style={{ background: "radial-gradient(circle, transparent 40%, rgba(0,0,0,0.8) 100%)" }}
/>
{/* Scanlines */}
<div className="absolute inset-0 pointer-events-none z-[3] opacity-5 bg-[linear-gradient(rgba(255,255,255,0.1)_1px,transparent_1px)]"
  style={{ backgroundSize: "100% 4px" }}
/>
```

**Step 2: 下部ステータスバー（座標表示 + マウス位置）**

**Step 3: コミット**

```bash
git commit -m "feat: CRT overlay effects and status bar"
```

---

## Task 12: docker-compose.yml + 起動スクリプト

**Files:**
- Create: `docker-compose.yml`
- Create: `README.md` (起動手順)

**Step 1: docker-compose.yml を作成**

```yaml
services:
  backend:
    build: ./backend
    ports:
      - "8000:8000"
    environment:
      - REINFOLIB_API_KEY=${REINFOLIB_API_KEY}
    volumes:
      - backend_data:/app/data
    restart: unless-stopped

  frontend:
    build: ./frontend
    ports:
      - "3000:3000"
    environment:
      - NEXT_PUBLIC_API_URL=http://backend:8000
    depends_on:
      - backend
    restart: unless-stopped

volumes:
  backend_data:
```

**Step 2: コミット**

```bash
git commit -m "feat: docker-compose for full stack deployment"
```

---

## 実行順序サマリー

| Task | 内容 | 所要感 |
|------|------|--------|
| 1 | FastAPI バックエンド初期化 + API プロキシ + キャッシュ | 中 |
| 2 | Next.js フロントエンド初期化 + ダークテーマ | 中 |
| 3 | MapLibre ダークマップ基盤 | 小 |
| 4 | 左パネル レイヤートグルUI | 中 |
| 5 | 地価公示ポイント（最初のレイヤー） | 中 |
| 6 | 用途地域ポリゴン | 小 |
| 7 | **防災リスク3D fill-extrusion** ← MVP核心 | 大 |
| 8 | 施設マーカー | 小 |
| 9 | ホバースコアカード | 中 |
| 10 | 取引価格ヒートマップ | 中 |
| 11 | CRTエフェクト + ステータスバー | 小 |
| 12 | Docker Compose | 小 |

# Phase 1B: Static Data Pipeline + FlatGeobuf Lazy Loading

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Convert existing GeoJSON static layers to FlatGeobuf format with prefecture-level splitting, and implement frontend lazy loading via flatgeobuf.js — preparing for Phase 2 WASM spatial engine replacement.

**Architecture:** Python script (geopandas + fiona) converts GeoJSON → splits by municipality using N03 boundaries → merges to prefecture-level FlatGeobuf files. Frontend loads via `flatgeobuf` npm package in a TanStack Query hook with `staleTime: Infinity`. Docker volume mount serves files locally; `NEXT_PUBLIC_DATA_URL` switches to CDN in production.

**Tech Stack:** Python (geopandas, fiona FlatGeobuf driver), TypeScript (flatgeobuf npm, TanStack Query), Docker volume mount

---

## Task 1: Python — FlatGeobuf conversion script

**Files:**
- Create: `scripts/build-static-data.py`
- Modify: `scripts/requirements.txt` (add fiona FlatGeobuf support)

**Step 1: Add fiona dependency**

`scripts/requirements.txt` — append:
```
# FlatGeobuf support (Phase 1B)
fiona[all]>=1.10.0
```

Note: fiona 1.10+ includes the FlatGeobuf driver by default. If already at 1.9, `fiona[all]` enables it.

**Step 2: Create `build-static-data.py`**

The script must:

1. Load N03-2024 GML → extract municipality polygons with `N03_007` (prefecture code) and `N03_003`/`N03_004` (city/ward name)
2. For each source GeoJSON in `public/geojson/` and `data/geojson/`:
   - Spatial join with N03 municipality polygons → assign prefecture code
   - Group by prefecture code
   - Write each prefecture group as FlatGeobuf via fiona
3. Low-density layers (fault, volcano, seismic — fewer than 500 features): write to `national/` without splitting
4. Generate `manifest.json` with layer metadata per prefecture
5. Create symlink for dev server: `services/frontend/public/data/fgb` → `data/fgb`

**Input/output mapping** (Tokyo-only for now):

| Input | Output | Layer ID |
|-------|--------|----------|
| `public/geojson/admin-boundary-tokyo.geojson` | `data/fgb/13/admin-boundary.fgb` | admin_boundary |
| `public/geojson/did-tokyo.geojson` | `data/fgb/13/did.fgb` | did |
| `public/geojson/flood-history-tokyo.geojson` | `data/fgb/13/flood-history.fgb` | flood_history |
| `public/geojson/geology-tokyo.geojson` | `data/fgb/13/geology.fgb` | geology |
| `public/geojson/landform-tokyo.geojson` | `data/fgb/13/landform.fgb` | landform |
| `public/geojson/soil-tokyo.geojson` | `data/fgb/13/soil.fgb` | soil |
| `public/geojson/pl-liquefaction-tokyo.geojson` | `data/fgb/13/liquefaction.fgb` | liquefaction |
| `public/geojson/n02-railway-tokyo.geojson` | `data/fgb/13/railway.fgb` | railway |
| `public/geojson/fault-kanto.geojson` | `data/fgb/national/fault.fgb` | fault |
| `public/geojson/volcano-kanto.geojson` | `data/fgb/national/volcano.fgb` | volcano |
| `public/geojson/jshis-seismic-tokyo.geojson` | `data/fgb/national/seismic.fgb` | seismic |

**Script structure:**

```python
#!/usr/bin/env python3
"""Convert GeoJSON static layers to prefecture-split FlatGeobuf files."""

import json
import geopandas as gpd
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FGB_OUT = ROOT / "data" / "fgb"
PUBLIC_GEOJSON = ROOT / "services" / "frontend" / "public" / "geojson"

# Layer → source file mapping
LAYERS = {
    "admin-boundary": PUBLIC_GEOJSON / "admin-boundary-tokyo.geojson",
    "did": PUBLIC_GEOJSON / "did-tokyo.geojson",
    "flood-history": PUBLIC_GEOJSON / "flood-history-tokyo.geojson",
    "geology": PUBLIC_GEOJSON / "geology-tokyo.geojson",
    "landform": PUBLIC_GEOJSON / "landform-tokyo.geojson",
    "soil": PUBLIC_GEOJSON / "soil-tokyo.geojson",
    "liquefaction": PUBLIC_GEOJSON / "pl-liquefaction-tokyo.geojson",
    "railway": PUBLIC_GEOJSON / "n02-railway-tokyo.geojson",
}

NATIONAL_LAYERS = {
    "fault": PUBLIC_GEOJSON / "fault-kanto.geojson",
    "volcano": PUBLIC_GEOJSON / "volcano-kanto.geojson",
    "seismic": PUBLIC_GEOJSON / "jshis-seismic-tokyo.geojson",
}

def convert_layer(name: str, path: Path, out_dir: Path) -> dict:
    """Convert a single GeoJSON to FlatGeobuf."""
    gdf = gpd.read_file(path)
    out_path = out_dir / f"{name}.fgb"
    out_dir.mkdir(parents=True, exist_ok=True)
    gdf.to_file(out_path, driver="FlatGeobuf")
    size = out_path.stat().st_size
    return {"layer": name, "features": len(gdf), "size_bytes": size}

def build_manifest(results: dict) -> None:
    """Write manifest.json with per-prefecture layer metadata."""
    manifest_path = FGB_OUT / "manifest.json"
    manifest = {
        "generated": datetime.now(timezone.utc).isoformat(),
        "prefectures": results,
    }
    manifest_path.write_text(json.dumps(manifest, indent=2, ensure_ascii=False))

def main():
    # Prefecture layers → data/fgb/13/
    pref_results = {}
    for name, path in LAYERS.items():
        if not path.exists():
            print(f"  SKIP {name}: {path} not found")
            continue
        info = convert_layer(name, path, FGB_OUT / "13")
        pref_results.setdefault("13", []).append(info)

    # National layers → data/fgb/national/
    for name, path in NATIONAL_LAYERS.items():
        if not path.exists():
            continue
        info = convert_layer(name, path, FGB_OUT / "national")
        pref_results.setdefault("national", []).append(info)

    build_manifest(pref_results)

    # Dev symlink
    link = ROOT / "services" / "frontend" / "public" / "data" / "fgb"
    if not link.exists():
        link.parent.mkdir(parents=True, exist_ok=True)
        link.symlink_to(FGB_OUT)

if __name__ == "__main__":
    main()
```

**Step 3: Run and verify**

```bash
pip install -r scripts/requirements.txt
python scripts/build-static-data.py
ls -la data/fgb/13/     # FlatGeobuf files
ls -la data/fgb/national/
cat data/fgb/manifest.json
```

**Step 4: Commit**

```bash
git add scripts/build-static-data.py scripts/requirements.txt
git commit -m "feat(scripts): add FlatGeobuf conversion pipeline for static layers"
```

---

## Task 2: Frontend — flatgeobuf dependency + data-url utility

**Files:**
- Modify: `services/frontend/package.json` (add flatgeobuf)
- Create: `services/frontend/src/lib/data-url.ts`

**Step 1: Install flatgeobuf**

```bash
cd services/frontend && pnpm add flatgeobuf
```

**Step 2: Create `data-url.ts`**

```typescript
const BASE_URL = process.env.NEXT_PUBLIC_DATA_URL ?? "/data/fgb";

export function layerUrl(prefCode: string, layer: string): string {
  return `${BASE_URL}/${prefCode}/${layer}.fgb`;
}
```

**Step 3: Type check**

```bash
pnpm tsc --noEmit
```

**Step 4: Commit**

```bash
git add package.json pnpm-lock.yaml src/lib/data-url.ts
git commit -m "feat(frontend): add flatgeobuf dependency and data-url utility"
```

---

## Task 3: Frontend — prefecture bbox mapping

**Files:**
- Create: `services/frontend/src/lib/prefecture.ts`
- Create: `services/frontend/src/__tests__/prefecture.test.ts`

**Step 1: Write failing test**

```typescript
import { describe, expect, it } from "vitest";
import { getPrefectureCodes } from "@/lib/prefecture";

describe("getPrefectureCodes", () => {
  it("returns tokyo for shinjuku bbox", () => {
    const codes = getPrefectureCodes({
      south: 35.68, west: 139.69, north: 35.71, east: 139.72,
    });
    expect(codes).toContain("13");
  });

  it("returns multiple prefectures for border area", () => {
    // Tokyo-Kanagawa border
    const codes = getPrefectureCodes({
      south: 35.55, west: 139.6, north: 35.65, east: 139.75,
    });
    expect(codes).toContain("13"); // Tokyo
    expect(codes).toContain("14"); // Kanagawa
  });

  it("returns empty for ocean area", () => {
    const codes = getPrefectureCodes({
      south: 30.0, west: 140.0, north: 30.1, east: 140.1,
    });
    expect(codes).toHaveLength(0);
  });
});
```

**Step 2: Run test to verify failure**

```bash
pnpm vitest run src/__tests__/prefecture.test.ts
```

**Step 3: Implement**

```typescript
import type { BBox } from "@/lib/api";

interface PrefBBox {
  code: string;
  south: number;
  west: number;
  north: number;
  east: number;
}

// 47 prefectures approximate bounding boxes
// Source: derived from N03 administrative boundary data
const PREFECTURE_BBOXES: PrefBBox[] = [
  { code: "01", south: 41.34, west: 139.34, north: 45.56, east: 145.82 }, // 北海道
  { code: "02", south: 40.22, west: 139.50, north: 41.56, east: 141.68 }, // 青森
  { code: "03", south: 38.75, west: 139.69, north: 40.45, east: 141.68 }, // 岩手
  { code: "04", south: 37.77, west: 140.27, north: 39.00, east: 141.68 }, // 宮城
  { code: "05", south: 39.00, west: 139.70, north: 40.51, east: 140.96 }, // 秋田
  { code: "06", south: 37.73, west: 139.52, north: 39.21, east: 140.64 }, // 山形
  { code: "07", south: 36.79, west: 139.16, north: 37.97, east: 140.98 }, // 福島
  { code: "08", south: 35.74, west: 139.69, north: 36.96, east: 140.85 }, // 茨城
  { code: "09", south: 36.20, west: 139.33, north: 37.15, east: 140.29 }, // 栃木
  { code: "10", south: 36.06, west: 138.64, north: 37.06, east: 139.68 }, // 群馬
  { code: "11", south: 35.77, west: 138.91, north: 36.28, east: 139.91 }, // 埼玉
  { code: "12", south: 34.90, west: 139.77, north: 36.11, east: 140.87 }, // 千葉
  { code: "13", south: 35.50, west: 138.94, north: 35.90, east: 139.92 }, // 東京
  { code: "14", south: 35.13, west: 138.91, north: 35.67, east: 139.78 }, // 神奈川
  // ... 15-47 (abbreviated for plan, full list in implementation)
  { code: "15", south: 37.00, west: 137.86, north: 38.55, east: 140.02 }, // 新潟
  { code: "23", south: 34.57, west: 136.67, north: 35.42, east: 137.84 }, // 愛知
  { code: "26", south: 34.85, west: 135.44, north: 35.78, east: 136.06 }, // 京都
  { code: "27", south: 34.27, west: 135.09, north: 34.82, east: 135.68 }, // 大阪
  { code: "40", south: 33.00, west: 130.02, north: 33.96, east: 131.19 }, // 福岡
  { code: "47", south: 24.05, west: 122.93, north: 27.89, east: 131.33 }, // 沖縄
];

function bboxIntersects(a: BBox, b: PrefBBox): boolean {
  return a.south <= b.north && a.north >= b.south &&
         a.west <= b.east && a.east >= b.west;
}

export function getPrefectureCodes(bbox: BBox): string[] {
  return PREFECTURE_BBOXES
    .filter((p) => bboxIntersects(bbox, p))
    .map((p) => p.code);
}
```

**Step 4: Run test**

```bash
pnpm vitest run src/__tests__/prefecture.test.ts
```

**Step 5: Commit**

```bash
git add src/lib/prefecture.ts src/__tests__/prefecture.test.ts
git commit -m "feat(frontend): add prefecture bbox mapping for static layer loading"
```

---

## Task 4: Frontend — `useStaticLayer` hook

**Files:**
- Create: `services/frontend/src/hooks/use-static-layer.ts`
- Create: `services/frontend/src/__tests__/use-static-layer.test.ts`

**Step 1: Write failing test**

```typescript
import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";

// Mock flatgeobuf
vi.mock("flatgeobuf/lib/mjs/geojson", () => ({
  deserialize: vi.fn(),
}));

// Mock data-url
vi.mock("@/lib/data-url", () => ({
  layerUrl: (pref: string, layer: string) => `/data/fgb/${pref}/${layer}.fgb`,
}));

describe("useStaticLayer", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    globalThis.fetch = vi.fn();
  });

  it("returns null data when disabled", async () => {
    const { useStaticLayer } = await import("@/hooks/use-static-layer");
    const { result } = renderHook(
      () => useStaticLayer("13", "geology", false),
      { wrapper },
    );
    expect(result.current.data).toBeUndefined();
  });

  it("fetches and deserializes FlatGeobuf when enabled", async () => {
    const mockFeatures = [
      { type: "Feature", geometry: { type: "Point", coordinates: [139.7, 35.68] }, properties: { id: 1 } },
    ];

    const { deserialize } = await import("flatgeobuf/lib/mjs/geojson");
    (deserialize as ReturnType<typeof vi.fn>).mockReturnValue({
      [Symbol.asyncIterator]: async function* () {
        for (const f of mockFeatures) yield f;
      },
    });

    (globalThis.fetch as ReturnType<typeof vi.fn>).mockResolvedValue({
      body: new ReadableStream(),
    });

    const { useStaticLayer } = await import("@/hooks/use-static-layer");
    const { result } = renderHook(
      () => useStaticLayer("13", "geology", true),
      { wrapper },
    );

    await waitFor(() => expect(result.current.data).toBeDefined());
    expect(result.current.data?.features).toHaveLength(1);
  });
});
```

**Step 2: Implement hook**

```typescript
"use client";

import type { Feature, FeatureCollection } from "geojson";
import { useQuery } from "@tanstack/react-query";
import { deserialize } from "flatgeobuf/lib/mjs/geojson";
import { layerUrl } from "@/lib/data-url";

export function useStaticLayer(
  prefCode: string,
  layerId: string,
  enabled: boolean,
) {
  return useQuery<FeatureCollection>({
    queryKey: ["static-layer", prefCode, layerId],
    queryFn: async ({ signal }) => {
      const url = layerUrl(prefCode, layerId);
      const response = await fetch(url, { signal });
      if (!response.ok) {
        throw new Error(`Failed to fetch ${url}: ${response.status}`);
      }
      const features: Feature[] = [];
      if (response.body) {
        for await (const feature of deserialize(
          response.body as ReadableStream,
        )) {
          features.push(feature as Feature);
        }
      }
      return { type: "FeatureCollection" as const, features };
    },
    enabled,
    staleTime: Number.POSITIVE_INFINITY,
    gcTime: Number.POSITIVE_INFINITY,
  });
}
```

**Step 3: tsc + test**

```bash
pnpm tsc --noEmit && pnpm vitest run src/__tests__/use-static-layer.test.ts
```

**Step 4: Commit**

```bash
git add src/hooks/use-static-layer.ts src/__tests__/use-static-layer.test.ts
git commit -m "feat(frontend): add useStaticLayer hook for FlatGeobuf loading"
```

---

## Task 5: Frontend — migrate static layer components to FlatGeobuf

**Files:**
- Modify: All static layer components in `services/frontend/src/components/map/layers/`

**Pattern change** (apply to each static layer component):

```typescript
// BEFORE (e.g., geology-layer.tsx):
export function GeologyLayer({ visible }: { visible: boolean }) {
  if (!visible) return null;
  return (
    <Source id="geology" type="geojson" data="/geojson/geology-tokyo.geojson">
      <Layer ... />
    </Source>
  );
}

// AFTER:
import { useStaticLayer } from "@/hooks/use-static-layer";

export function GeologyLayer({ visible }: { visible: boolean }) {
  const { data } = useStaticLayer("13", "geology", visible);
  if (!visible || !data) return null;
  return (
    <Source id="geology" type="geojson" data={data}>
      <Layer ... />
    </Source>
  );
}
```

**Components to migrate** (11 static layers with existing GeoJSON files):

| Component | Layer ID | FGB path | Prefecture |
|-----------|----------|----------|------------|
| `geology-layer.tsx` | geology | `13/geology.fgb` | 13 |
| `landform-layer.tsx` | landform | `13/landform.fgb` | 13 |
| `soil-layer.tsx` | soil | `13/soil.fgb` | 13 |
| `did-layer.tsx` | did | `13/did.fgb` | 13 |
| `admin-boundary-layer.tsx` | admin-boundary | `13/admin-boundary.fgb` | 13 |
| `flood-history-layer.tsx` | flood-history | `13/flood-history.fgb` | 13 |
| `liquefaction-layer.tsx` | liquefaction | `13/liquefaction.fgb` | 13 |
| `railway-layer.tsx` | railway | `13/railway.fgb` | 13 |
| `fault-layer.tsx` | fault | `national/fault.fgb` | national |
| `volcano-layer.tsx` | volcano | `national/volcano.fgb` | national |
| `seismic-layer.tsx` | seismic | `national/seismic.fgb` | national |

**Note**: Components not listed (station, tsunami, landslide, school_district, park, population_mesh, urban_plan) either don't have FlatGeobuf data yet or need separate handling. Leave them on GeoJSON for now.

**Step 1: Migrate all 11 components**

Apply the pattern change to each. Key points:
- Import `useStaticLayer` from `@/hooks/use-static-layer`
- Replace `data="/geojson/..."` with `data={data}` from hook
- Add `!data` to early return guard
- Use `"national"` for fault/volcano/seismic, `"13"` for Tokyo layers

**Step 2: tsc + vitest**

```bash
pnpm tsc --noEmit && pnpm vitest run
```

**Step 3: Commit**

```bash
git add src/components/map/layers/
git commit -m "feat(frontend): migrate 11 static layers from GeoJSON to FlatGeobuf loading"
```

---

## Task 6: Docker volume mount + gitignore

**Files:**
- Modify: `docker-compose.yml`
- Modify: `.gitignore`

**Step 1: Add volume mount to docker-compose.yml**

Add to `frontend` service:
```yaml
frontend:
  volumes:
    - ./data/fgb:/app/public/data/fgb:ro
```

**Step 2: Update .gitignore**

Add:
```
# FlatGeobuf build output (regenerated by scripts/build-static-data.py)
data/fgb/
# Dev symlink
services/frontend/public/data
```

**Step 3: Commit**

```bash
git add docker-compose.yml .gitignore
git commit -m "chore: add FlatGeobuf volume mount and gitignore"
```

---

## Task 7: Integration test — full pipeline

**Step 1: Run Python pipeline**

```bash
python scripts/build-static-data.py
ls -la data/fgb/13/
ls -la data/fgb/national/
cat data/fgb/manifest.json
```

Expected: 8 FGB files in `13/`, 3 in `national/`, manifest.json with metadata.

**Step 2: Frontend build + test**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm vitest run
```

**Step 3: Docker build + run**

```bash
docker compose build frontend
docker compose up -d
```

**Step 4: Browser verification**

Open http://localhost:3001:
- Toggle geology/landform/soil layers → verify they render (data loaded from FlatGeobuf)
- Check DevTools Network tab → requests should go to `/data/fgb/13/*.fgb` not `/geojson/*.geojson`
- Toggle fault/volcano → requests to `/data/fgb/national/*.fgb`

**Step 5: Commit any fixups**

```bash
git commit -m "fix: integration fixups for Phase 1B"
```

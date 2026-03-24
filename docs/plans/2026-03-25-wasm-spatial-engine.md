# Phase 2a: WASM Spatial Engine Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust WASM spatial engine (R-tree + FlatGeobuf reader) running in a Web Worker, replacing flatgeobuf.js for static layer loading with viewport-aware spatial filtering.

**Architecture:** Rust crate (`services/wasm/`) compiled to WASM via wasm-pack. Web Worker fetches all 11 FlatGeobuf files on page load, passes bytes to WASM which builds R-trees per layer. On viewport change, WASM queries R-tree → returns GeoJSON string for viewport features only. Fallback to flatgeobuf.js on WASM failure.

**Tech Stack:** Rust (rstar 0.12, flatgeobuf 6.0, wasm-bindgen 0.2, geozero 0.14), TypeScript (Web Worker, TanStack Query)

**Verified:** rstar + flatgeobuf build for `wasm32-unknown-unknown`. Binary size ~95KB uncompressed.

---

## Task 1: Rust WASM crate scaffold + FlatGeobuf reader

**Files:**
- Create: `services/wasm/Cargo.toml`
- Create: `services/wasm/src/lib.rs`
- Create: `services/wasm/src/fgb_reader.rs`
- Create: `scripts/build-wasm.sh`

**Step 1: Create Cargo.toml**

```toml
[package]
name = "realestate-wasm"
version = "0.1.0"
edition = "2024"
rust-version = "1.94"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
realestate-geo-math = { path = "../backend/lib/geo-math" }
wasm-bindgen = "0.2"
js-sys = "0.3"
rstar = "0.12"
flatgeobuf = { version = "6", default-features = false }
geozero = "0.14"
serde_json = "1"

[profile.release]
opt-level = "s"
lto = true
strip = true
```

**Step 2: Create fgb_reader.rs**

Module that reads FlatGeobuf bytes and extracts features with coordinates + properties:

```rust
use std::io::Cursor;
use flatgeobuf::FgbReader;
use geozero::geojson::GeoJsonWriter;
use geozero::GeozeroDatasource;

pub struct ParsedFeature {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub geojson: String,  // Single feature as GeoJSON string
}

/// Parse FlatGeobuf bytes into a Vec of features with bbox for R-tree insertion.
pub fn parse_fgb(bytes: &[u8]) -> Result<Vec<ParsedFeature>, String> {
    let mut cursor = Cursor::new(bytes);
    let reader = FgbReader::open(&mut cursor).map_err(|e| e.to_string())?;
    let mut iter = reader.select_all_seq().map_err(|e| e.to_string())?;
    let mut features = Vec::new();

    while let Some(feature) = iter.next().map_err(|e| e.to_string())? {
        // Extract bbox from geometry
        let geom = feature.geometry().ok_or("no geometry")?;
        // ... extract envelope (min_x, min_y, max_x, max_y)

        // Serialize feature to GeoJSON string
        let mut json = Vec::new();
        let mut writer = GeoJsonWriter::new(&mut json);
        feature.process(&mut writer).map_err(|e| e.to_string())?;

        features.push(ParsedFeature {
            min_x, min_y, max_x, max_y,
            geojson: String::from_utf8(json).map_err(|e| e.to_string())?,
        });
    }
    Ok(features)
}
```

Note: The exact geozero/flatgeobuf API for extracting bbox and serializing features will need to be figured out during implementation. The key interfaces are:
- `FgbReader::open(&mut Cursor<&[u8]>)` — opens from byte slice
- `reader.select_all_seq()` — sequential iteration (no HTTP)
- `feature.geometry()` — access raw geometry
- `GeoJsonWriter` from geozero — serialize feature to GeoJSON string

**Step 3: Create lib.rs (minimal, just re-export)**

```rust
mod fgb_reader;
pub use fgb_reader::parse_fgb;
```

**Step 4: Create build script**

```bash
#!/usr/bin/env bash
# scripts/build-wasm.sh
set -euo pipefail
cd "$(dirname "$0")/../services/wasm"
wasm-pack build --target web --out-dir ../../services/frontend/public/wasm --release
echo "WASM built: $(ls -la ../../services/frontend/public/wasm/*.wasm | awk '{print $5}') bytes"
```

**Step 5: Build and test**

```bash
cd services/wasm && cargo test  # native tests
bash scripts/build-wasm.sh       # WASM build
ls -la services/frontend/public/wasm/
```

**Step 6: Commit**

```bash
git add services/wasm/ scripts/build-wasm.sh
git commit -m "feat(wasm): scaffold WASM crate with FlatGeobuf reader"
```

---

## Task 2: R-tree spatial index + SpatialEngine API

**Files:**
- Create: `services/wasm/src/spatial_index.rs`
- Modify: `services/wasm/src/lib.rs` (add wasm_bindgen exports)

**Step 1: Create spatial_index.rs**

```rust
use rstar::{RTree, AABB, RTreeObject, PointDistance};
use std::collections::HashMap;

/// A feature stored in the R-tree with its bbox and index.
struct IndexedFeature {
    index: u32,
    envelope: AABB<[f64; 2]>,
}

impl RTreeObject for IndexedFeature {
    type Envelope = AABB<[f64; 2]>;
    fn envelope(&self) -> Self::Envelope { self.envelope }
}

impl PointDistance for IndexedFeature {
    fn distance_2(&self, point: &[f64; 2]) -> f64 {
        self.envelope.distance_2(point)
    }
}

pub struct LayerIndex {
    tree: RTree<IndexedFeature>,
    features_json: Vec<String>,  // GeoJSON strings per feature
}

impl LayerIndex {
    pub fn from_parsed(features: Vec<ParsedFeature>) -> Self {
        let mut items = Vec::with_capacity(features.len());
        let mut json = Vec::with_capacity(features.len());

        for (i, f) in features.into_iter().enumerate() {
            items.push(IndexedFeature {
                index: i as u32,
                envelope: AABB::from_corners(
                    [f.min_x, f.min_y],
                    [f.max_x, f.max_y],
                ),
            });
            json.push(f.geojson);
        }

        Self {
            tree: RTree::bulk_load(items),
            features_json: json,
        }
    }

    pub fn query_bbox(&self, south: f64, west: f64, north: f64, east: f64) -> Vec<u32> {
        let envelope = AABB::from_corners([west, south], [east, north]);
        self.tree
            .locate_in_envelope(&envelope)
            .map(|f| f.index)
            .collect()
    }

    pub fn get_features_geojson(&self, indices: &[u32]) -> String {
        let features: Vec<&str> = indices.iter()
            .filter_map(|&i| self.features_json.get(i as usize).map(|s| s.as_str()))
            .collect();
        format!(r#"{{"type":"FeatureCollection","features":[{}]}}"#, features.join(","))
    }

    pub fn feature_count(&self) -> u32 {
        self.features_json.len() as u32
    }
}
```

**Step 2: Update lib.rs with wasm_bindgen exports**

```rust
mod fgb_reader;
mod spatial_index;

use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use spatial_index::LayerIndex;

#[wasm_bindgen]
pub struct SpatialEngine {
    layers: HashMap<String, LayerIndex>,
}

#[wasm_bindgen]
impl SpatialEngine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { layers: HashMap::new() }
    }

    /// Load a FlatGeobuf layer from raw bytes. Returns feature count.
    pub fn load_layer(&mut self, layer_id: &str, fgb_bytes: &[u8]) -> Result<u32, JsValue> {
        let features = fgb_reader::parse_fgb(fgb_bytes)
            .map_err(|e| JsValue::from_str(&e))?;
        let count = features.len() as u32;
        let index = LayerIndex::from_parsed(features);
        self.layers.insert(layer_id.to_string(), index);
        Ok(count)
    }

    /// Query features within bbox. Returns GeoJSON FeatureCollection string.
    pub fn query(&self, layer_id: &str, south: f64, west: f64, north: f64, east: f64) -> Result<String, JsValue> {
        let index = self.layers.get(layer_id)
            .ok_or_else(|| JsValue::from_str(&format!("layer not found: {layer_id}")))?;
        let indices = index.query_bbox(south, west, north, east);
        Ok(index.get_features_geojson(&indices))
    }

    /// Query multiple layers at once. Returns JSON object keyed by layer.
    pub fn query_layers(&self, layer_ids: &str, south: f64, west: f64, north: f64, east: f64) -> Result<String, JsValue> {
        let mut results = Vec::new();
        for layer_id in layer_ids.split(',') {
            let layer_id = layer_id.trim();
            if let Some(index) = self.layers.get(layer_id) {
                let indices = index.query_bbox(south, west, north, east);
                let fc = index.get_features_geojson(&indices);
                results.push(format!(r#""{layer_id}":{fc}"#));
            }
        }
        Ok(format!("{{{}}}", results.join(",")))
    }

    pub fn feature_count(&self, layer_id: &str) -> u32 {
        self.layers.get(layer_id).map(|l| l.feature_count()).unwrap_or(0)
    }

    pub fn loaded_layers(&self) -> String {
        let ids: Vec<&str> = self.layers.keys().map(|s| s.as_str()).collect();
        ids.join(",")
    }
}
```

**Step 3: Write Rust tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Generate a minimal FGB file in test, or test with real file
    #[test]
    fn test_engine_lifecycle() {
        let mut engine = SpatialEngine::new();
        assert_eq!(engine.loaded_layers(), "");
        // ... load test FGB, query bbox, verify results
    }
}
```

For integration tests, use the actual FGB files at `data/fgb/13/geology.fgb`.

**Step 4: Build + test**

```bash
cd services/wasm && cargo test
bash scripts/build-wasm.sh
```

**Step 5: Commit**

```bash
git add services/wasm/
git commit -m "feat(wasm): add R-tree spatial index and SpatialEngine API"
```

---

## Task 3: Web Worker

**Files:**
- Create: `services/frontend/src/lib/wasm/worker.ts`

**Step 1: Create worker**

```typescript
/// Web Worker: manages WASM SpatialEngine lifecycle
/// Protocol:
///   Main → Worker: { type: "init", layers: [{id, url}] }
///   Main → Worker: { type: "query", id, bbox: {south,west,north,east}, layers: string[] }
///   Worker → Main: { type: "init-done", counts: Record<string, number> }
///   Worker → Main: { type: "query-result", id, geojson: string }
///   Worker → Main: { type: "error", message: string }

import init, { SpatialEngine } from "/wasm/realestate_wasm.js";

let engine: SpatialEngine | null = null;

interface InitMsg {
  type: "init";
  layers: { id: string; url: string }[];
}

interface QueryMsg {
  type: "query";
  id: number;
  bbox: { south: number; west: number; north: number; east: number };
  layers: string[];
}

self.onmessage = async (e: MessageEvent<InitMsg | QueryMsg>) => {
  try {
    if (e.data.type === "init") {
      await init();  // Initialize WASM module
      engine = new SpatialEngine();

      const counts: Record<string, number> = {};

      // Fetch all FGB files in parallel
      const results = await Promise.allSettled(
        e.data.layers.map(async ({ id, url }) => {
          const resp = await fetch(url);
          if (!resp.ok) throw new Error(`${url}: ${resp.status}`);
          const bytes = new Uint8Array(await resp.arrayBuffer());
          const count = engine!.load_layer(id, bytes);
          counts[id] = count;
        }),
      );

      // Log failures but don't block
      for (const r of results) {
        if (r.status === "rejected") {
          console.warn("[wasm-worker] layer load failed:", r.reason);
        }
      }

      self.postMessage({ type: "init-done", counts });

    } else if (e.data.type === "query") {
      if (!engine) {
        self.postMessage({ type: "error", message: "Engine not initialized" });
        return;
      }
      const { id, bbox, layers } = e.data;
      const geojson = engine.query_layers(
        layers.join(","),
        bbox.south, bbox.west, bbox.north, bbox.east,
      );
      self.postMessage({ type: "query-result", id, geojson });
    }
  } catch (err) {
    self.postMessage({ type: "error", message: String(err) });
  }
};
```

**Step 2: Type check**

```bash
pnpm tsc --noEmit
```

Note: Worker imports from `/wasm/realestate_wasm.js` which is a runtime path. TypeScript may need a type declaration for the WASM module. Create `src/lib/wasm/wasm.d.ts`:

```typescript
declare module "/wasm/realestate_wasm.js" {
  export default function init(): Promise<void>;
  export class SpatialEngine {
    constructor();
    load_layer(layer_id: string, fgb_bytes: Uint8Array): number;
    query(layer_id: string, south: number, west: number, north: number, east: number): string;
    query_layers(layer_ids: string, south: number, west: number, north: number, east: number): string;
    feature_count(layer_id: string): number;
    loaded_layers(): string;
  }
}
```

**Step 3: Commit**

```bash
git add src/lib/wasm/
git commit -m "feat(frontend): add WASM spatial engine Web Worker"
```

---

## Task 4: SpatialEngine adapter (main thread)

**Files:**
- Create: `services/frontend/src/lib/wasm/spatial-engine.ts`

**Step 1: Create adapter**

```typescript
import type { FeatureCollection } from "geojson";
import type { BBox } from "@/lib/api";
import { layerUrl } from "@/lib/data-url";

// Layer config for WASM loading
const WASM_LAYERS = [
  // Prefecture 13 (Tokyo)
  { id: "admin-boundary", url: layerUrl("13", "admin-boundary") },
  { id: "did", url: layerUrl("13", "did") },
  { id: "flood-history", url: layerUrl("13", "flood-history") },
  { id: "geology", url: layerUrl("13", "geology") },
  { id: "landform", url: layerUrl("13", "landform") },
  { id: "liquefaction", url: layerUrl("13", "liquefaction") },
  { id: "railway", url: layerUrl("13", "railway") },
  { id: "soil", url: layerUrl("13", "soil") },
  // National
  { id: "fault", url: layerUrl("national", "fault") },
  { id: "volcano", url: layerUrl("national", "volcano") },
  { id: "seismic", url: layerUrl("national", "seismic") },
];

type Listener = (ready: boolean) => void;

class SpatialEngineAdapter {
  private worker: Worker | null = null;
  private _ready = false;
  private listeners: Listener[] = [];
  private nextId = 0;
  private pending = new Map<number, { resolve: (v: string) => void; reject: (e: Error) => void }>();
  private layerCounts: Record<string, number> = {};

  get ready(): boolean { return this._ready; }

  async init(): Promise<void> {
    if (typeof window === "undefined") return; // SSR guard
    if (this.worker) return; // already initialized

    try {
      this.worker = new Worker(
        new URL("./worker.ts", import.meta.url),
        { type: "module" },
      );
      this.worker.onmessage = this.handleMessage.bind(this);
      this.worker.onerror = (e) => console.error("[spatial-engine] worker error:", e);

      await new Promise<void>((resolve, reject) => {
        const timeout = setTimeout(() => reject(new Error("WASM init timeout")), 30_000);
        const handler = (e: MessageEvent) => {
          if (e.data.type === "init-done") {
            clearTimeout(timeout);
            this.layerCounts = e.data.counts;
            this._ready = true;
            this.listeners.forEach((l) => l(true));
            resolve();
          } else if (e.data.type === "error") {
            clearTimeout(timeout);
            reject(new Error(e.data.message));
          }
        };
        this.worker!.addEventListener("message", handler, { once: false });
        this.worker!.postMessage({ type: "init", layers: WASM_LAYERS });
      });
    } catch (err) {
      console.warn("[spatial-engine] WASM init failed, falling back to JS:", err);
      this.worker?.terminate();
      this.worker = null;
    }
  }

  async query(bbox: BBox, layers: string[]): Promise<FeatureCollection> {
    if (!this.worker || !this._ready) {
      throw new Error("SpatialEngine not ready");
    }

    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      this.pending.set(id, {
        resolve: (geojson) => {
          const fc = JSON.parse(geojson) as FeatureCollection;
          resolve(fc);
        },
        reject,
      });
      this.worker!.postMessage({ type: "query", id, bbox, layers });
    });
  }

  onReady(listener: Listener): () => void {
    this.listeners.push(listener);
    if (this._ready) listener(true);
    return () => {
      this.listeners = this.listeners.filter((l) => l !== listener);
    };
  }

  private handleMessage(e: MessageEvent): void {
    if (e.data.type === "query-result") {
      const p = this.pending.get(e.data.id);
      if (p) {
        this.pending.delete(e.data.id);
        p.resolve(e.data.geojson);
      }
    } else if (e.data.type === "error") {
      // Reject all pending queries
      for (const [, p] of this.pending) {
        p.reject(new Error(e.data.message));
      }
      this.pending.clear();
    }
  }

  dispose(): void {
    this.worker?.terminate();
    this.worker = null;
    this._ready = false;
  }
}

export const spatialEngine = new SpatialEngineAdapter();
```

**Step 2: tsc**

```bash
pnpm tsc --noEmit
```

**Step 3: Commit**

```bash
git add src/lib/wasm/spatial-engine.ts
git commit -m "feat(frontend): add SpatialEngine main thread adapter"
```

---

## Task 5: Update useStaticLayer hook — WASM path + fallback

**Files:**
- Modify: `services/frontend/src/hooks/use-static-layer.ts`
- Create: `services/frontend/src/hooks/use-spatial-engine.ts`

**Step 1: Create engine readiness hook**

```typescript
// use-spatial-engine.ts
"use client";
import { useEffect, useState } from "react";
import { spatialEngine } from "@/lib/wasm/spatial-engine";

export function useSpatialEngineReady(): boolean {
  const [ready, setReady] = useState(spatialEngine.ready);
  useEffect(() => spatialEngine.onReady(setReady), []);
  return ready;
}
```

**Step 2: Update useStaticLayer with dual path**

```typescript
// use-static-layer.ts — updated
"use client";

import type { Feature, FeatureCollection } from "geojson";
import { useQuery } from "@tanstack/react-query";
import { deserialize } from "flatgeobuf/lib/mjs/geojson";
import { layerUrl } from "@/lib/data-url";
import { spatialEngine } from "@/lib/wasm/spatial-engine";
import { useSpatialEngineReady } from "@/hooks/use-spatial-engine";
import { useMapStore } from "@/stores/map-store";

export function useStaticLayer(
  prefCode: string,
  layerId: string,
  enabled: boolean,
) {
  const wasmReady = useSpatialEngineReady();
  const bbox = useMapStore((s) => s.getBBox());

  // WASM path: R-tree query with current bbox
  const wasmQuery = useQuery<FeatureCollection>({
    queryKey: ["static-layer-wasm", layerId, bbox.south, bbox.west, bbox.north, bbox.east],
    queryFn: async () => spatialEngine.query(bbox, [layerId]),
    enabled: enabled && wasmReady,
    staleTime: 5_000,  // Re-query after 5s stale (bbox-dependent)
  });

  // Fallback path: full FlatGeobuf load (no bbox filtering)
  const fallbackQuery = useQuery<FeatureCollection>({
    queryKey: ["static-layer-fallback", prefCode, layerId],
    queryFn: async ({ signal }) => {
      const url = layerUrl(prefCode, layerId);
      const response = await fetch(url, { signal });
      if (!response.ok) throw new Error(`Failed to fetch ${url}: ${response.status}`);
      const features: Feature[] = [];
      if (response.body) {
        for await (const feature of deserialize(response.body as ReadableStream)) {
          features.push(feature as Feature);
        }
      }
      return { type: "FeatureCollection" as const, features };
    },
    enabled: enabled && !wasmReady,  // Only when WASM not available
    staleTime: Number.POSITIVE_INFINITY,
    gcTime: Number.POSITIVE_INFINITY,
  });

  return wasmReady ? wasmQuery : fallbackQuery;
}
```

**Step 3: Initialize WASM engine in layout or page**

Add to `src/app/page.tsx` (inside component, before return):

```typescript
useEffect(() => {
  spatialEngine.init();
  return () => spatialEngine.dispose();
}, []);
```

**Step 4: tsc + vitest**

```bash
pnpm tsc --noEmit && pnpm vitest run
```

**Step 5: Commit**

```bash
git add src/hooks/ src/lib/wasm/ src/app/page.tsx
git commit -m "feat(frontend): integrate WASM spatial engine with fallback to flatgeobuf.js"
```

---

## Task 6: Build pipeline + Docker + gitignore

**Files:**
- Modify: `services/frontend/.gitignore` (add `public/wasm/`)
- Modify: `docker-compose.yml` (add WASM volume mount)
- Modify: `services/frontend/package.json` (add prebuild script)

**Step 1: gitignore**

Add to `services/frontend/.gitignore`:
```
public/wasm/
```

**Step 2: Docker volume**

Add to `docker-compose.yml` frontend volumes:
```yaml
- ./services/frontend/public/wasm:/app/public/wasm:ro
```

**Step 3: package.json scripts**

Add:
```json
"prebuild": "bash ../../scripts/build-wasm.sh",
"build:wasm": "bash ../../scripts/build-wasm.sh"
```

**Step 4: Commit**

```bash
git add .gitignore docker-compose.yml package.json
git commit -m "chore: add WASM build pipeline, volume mount, gitignore"
```

---

## Task 7: Integration test — full pipeline

**Step 1: Build WASM**

```bash
bash scripts/build-wasm.sh
ls -la services/frontend/public/wasm/
```

Expected: `realestate_wasm_bg.wasm` (~100-200KB), `realestate_wasm.js`, `realestate_wasm.d.ts`

**Step 2: Rust tests**

```bash
cd services/wasm && cargo test
```

**Step 3: Frontend tests**

```bash
cd services/frontend && pnpm tsc --noEmit && pnpm vitest run
```

**Step 4: Docker build + run**

```bash
docker compose build frontend
docker compose up -d
```

**Step 5: Browser verification**

Open http://localhost:3001:
- Open DevTools Console → look for `[spatial-engine]` or `[wasm-worker]` init messages
- Toggle a static layer (geology) → verify it renders
- Pan/zoom → verify data updates for viewport (not full dataset)
- Check Network tab → FGB files fetched once at init, no subsequent FGB requests on pan
- Check Performance → R-tree query should be < 1ms per viewport change

**Step 6: Commit any fixups**

```bash
git commit -m "fix: integration fixups for Phase 2a WASM spatial engine"
```

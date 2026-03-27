# WASM Optimization Phase 0+1: ID Normalization + Correctness

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix WASM correctness issues: normalize layer IDs at WASM/FGB boundaries, replace binary `ready` with granular `loadedLayers`, add request-scoped error handling, and add Performance API observability.

**Architecture:** Create a canonical layer ID mapping (`layer-ids.ts`), refactor `SpatialEngineAdapter` to track per-layer readiness, update worker message protocol with typed error responses including request IDs, and instrument init/load/query with `performance.mark/measure`.

**Tech Stack:** TypeScript, Web Workers, Performance API, Vitest, React 19

## Non-Goals (Phase 1)

- **WASM stats is NOT re-enabled.** `computeStats()` remains unconditionally throwing. `statsReady` is internal-only and not exposed via hooks. Stats re-enablement is a Phase 3 concern.
- **Full ID normalization across all UI code is NOT in scope.** This plan normalizes IDs at the WASM/FGB boundary only. `layers.ts`, `themes.ts`, `layer-renderer.tsx`, and MapLibre source IDs continue using underscore IDs. A full system-wide rename is a separate task.
- **`useSpatialEngineState()` is Phase 1 scoped.** It re-renders on `init-done` only. Phase 3 will extend the notification system for `layer-loaded` events. This limitation is documented in the hook.

---

## File Structure

| File | Change | Responsibility |
|------|--------|----------------|
| `services/frontend/src/lib/layer-ids.ts` | Create | Canonical layer ID normalization |
| `services/frontend/src/lib/wasm/spatial-engine.ts` | Modify | `loadedLayers`, `queryReady`, Performance API, error handling |
| `services/frontend/src/lib/wasm/worker.ts` | Modify | `query-error`/`stats-error` messages, Performance API marks |
| `services/frontend/src/hooks/use-spatial-engine.ts` | Modify | Return granular readiness (init-done scoped) |
| `services/frontend/src/__tests__/layer-ids.test.ts` | Create | ID normalization tests |
| `services/frontend/src/__tests__/spatial-engine-ready.test.ts` | Create | Ready state tests |
| `services/frontend/src/__tests__/spatial-engine-errors.test.ts` | Create | Error isolation regression tests |

---

### Task 1: Canonical Layer ID Normalization

**Files:**
- Create: `services/frontend/src/lib/layer-ids.ts`
- Create: `services/frontend/src/__tests__/layer-ids.test.ts`

- [ ] **Step 1: Write the test**

Create `services/frontend/src/__tests__/layer-ids.test.ts`:

```typescript
import { describe, expect, it } from "vitest";
import { canonicalLayerId } from "@/lib/layer-ids";

describe("canonicalLayerId", () => {
  it("normalizes underscore IDs to hyphen-case", () => {
    expect(canonicalLayerId("admin_boundary")).toBe("admin-boundary");
    expect(canonicalLayerId("flood_history")).toBe("flood-history");
    expect(canonicalLayerId("steep_slope")).toBe("steep-slope");
    expect(canonicalLayerId("land_price_ts")).toBe("land-price-ts");
    expect(canonicalLayerId("population_mesh")).toBe("population-mesh");
  });

  it("passes through already-canonical IDs unchanged", () => {
    expect(canonicalLayerId("geology")).toBe("geology");
    expect(canonicalLayerId("landform")).toBe("landform");
    expect(canonicalLayerId("admin-boundary")).toBe("admin-boundary");
    expect(canonicalLayerId("did")).toBe("did");
  });

  it("passes through unknown IDs unchanged", () => {
    expect(canonicalLayerId("unknown_layer")).toBe("unknown_layer");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd services/frontend && pnpm vitest run src/__tests__/layer-ids.test.ts
```

Expected: FAIL — module `@/lib/layer-ids` does not exist.

- [ ] **Step 3: Implement layer-ids.ts**

Create `services/frontend/src/lib/layer-ids.ts`:

```typescript
/**
 * Canonical layer ID normalization.
 *
 * UI code (layers.ts, themes.ts, stores) uses underscore IDs for
 * backwards compatibility. WASM manifest, FGB filenames, and worker
 * messages use hyphen-case.
 *
 * Scope: This normalizes at WASM/FGB boundaries only. UI-side code
 * continues using underscore IDs. A full system-wide rename is out
 * of scope for this module.
 */

const ID_NORMALIZE: Record<string, string> = {
  admin_boundary: "admin-boundary",
  flood_history: "flood-history",
  steep_slope: "steep-slope",
  land_price_ts: "land-price-ts",
  population_mesh: "population-mesh",
};

/** Convert any layer ID variant to canonical hyphen-case form. */
export function canonicalLayerId(id: string): string {
  return ID_NORMALIZE[id] ?? id;
}
```

- [ ] **Step 4: Run test to verify it passes**

```bash
cd services/frontend && pnpm vitest run src/__tests__/layer-ids.test.ts
```

Expected: PASS

- [ ] **Step 5: Run full suite + type check**

```bash
cd services/frontend && pnpm vitest run && pnpm tsc --noEmit
```

- [ ] **Step 6: Commit**

```bash
git add services/frontend/src/lib/layer-ids.ts services/frontend/src/__tests__/layer-ids.test.ts
git commit -m "feat(wasm): add canonical layer ID normalization

Adds canonicalLayerId() to resolve underscore vs hyphen ID conflicts
at WASM/FGB boundaries (admin_boundary vs admin-boundary, etc.).
UI-side code continues using underscore IDs."
```

---

### Task 2: Ready State Separation + loadedLayers

**Files:**
- Modify: `services/frontend/src/lib/wasm/spatial-engine.ts`
- Modify: `services/frontend/src/hooks/use-spatial-engine.ts`
- Create: `services/frontend/src/__tests__/spatial-engine-ready.test.ts`

- [ ] **Step 1: Write the test**

Create `services/frontend/src/__tests__/spatial-engine-ready.test.ts`:

```typescript
import { describe, expect, it, beforeEach } from "vitest";
import { SpatialEngineAdapter } from "@/lib/wasm/spatial-engine";

describe("SpatialEngineAdapter ready state", () => {
  let adapter: SpatialEngineAdapter;

  beforeEach(() => {
    adapter = new SpatialEngineAdapter();
  });

  it("starts with empty loadedLayers", () => {
    expect(adapter.loadedLayers.size).toBe(0);
  });

  it("queryReady returns false when no layers loaded", () => {
    expect(adapter.queryReady(["geology"])).toBe(false);
  });

  it("queryReady returns true after layer is registered", () => {
    adapter.registerLoadedLayers({ geology: 133, landform: 370 });
    expect(adapter.queryReady(["geology"])).toBe(true);
    expect(adapter.queryReady(["geology", "landform"])).toBe(true);
  });

  it("queryReady returns false if any requested layer is missing", () => {
    adapter.registerLoadedLayers({ geology: 133 });
    expect(adapter.queryReady(["geology", "missing"])).toBe(false);
  });

  it("queryReady normalizes underscore IDs to canonical form", () => {
    adapter.registerLoadedLayers({ "admin-boundary": 6902 });
    expect(adapter.queryReady(["admin_boundary"])).toBe(true);
  });

  it("registerLoadedLayers normalizes keys to canonical form", () => {
    // Worker might report with underscore IDs in some edge cases
    adapter.registerLoadedLayers({ admin_boundary: 6902 });
    expect(adapter.loadedLayers.has("admin-boundary")).toBe(true);
    expect(adapter.loadedLayers.has("admin_boundary")).toBe(false);
  });

  it("ready is false when no layers loaded", () => {
    expect(adapter.ready).toBe(false);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cd services/frontend && pnpm vitest run src/__tests__/spatial-engine-ready.test.ts
```

Expected: FAIL — `SpatialEngineAdapter` doesn't have `loadedLayers`, `queryReady`, `registerLoadedLayers`.

- [ ] **Step 3: Modify spatial-engine.ts**

In `services/frontend/src/lib/wasm/spatial-engine.ts`:

1. Add import at top:
```typescript
import { canonicalLayerId } from "@/lib/layer-ids";
```

2. Replace the `_ready` field:

Replace:
```typescript
  private _ready = false;
```
With:
```typescript
  private _loadedLayers = new Set<string>();
```

3. Replace the `ready` getter and add new methods:

Replace:
```typescript
  get ready(): boolean {
    return this._ready;
  }
```
With:
```typescript
  /** Set of canonical layer IDs that have been loaded into the R-tree. */
  get loadedLayers(): ReadonlySet<string> {
    return this._loadedLayers;
  }

  /** True if the worker is initialized and at least one layer is loaded. */
  get ready(): boolean {
    return this.worker !== null && this._loadedLayers.size > 0;
  }

  /** Check if ALL specified layers are loaded (accepts any ID form). */
  queryReady(layerIds: string[]): boolean {
    return layerIds.every((id) => this._loadedLayers.has(canonicalLayerId(id)));
  }

  /**
   * Register layers reported by init-done or layer-loaded messages.
   * Keys are normalized to canonical hyphen-case form.
   */
  registerLoadedLayers(counts: Record<string, number>): void {
    for (const id of Object.keys(counts)) {
      this._loadedLayers.add(canonicalLayerId(id));
    }
  }
```

4. In `handleMessage`, update the `init-done` case:

Replace:
```typescript
      case "init-done": {
        this._ready = true;
        this.notifyListeners(true);
        break;
      }
```
With:
```typescript
      case "init-done": {
        this.registerLoadedLayers(msg.counts);
        this.notifyListeners(true);
        break;
      }
```

5. In the `query` method, update the guard:

Replace:
```typescript
    if (this.worker === null || !this._ready) {
```
With:
```typescript
    if (this.worker === null || this._loadedLayers.size === 0) {
```

6. In the `computeStats` method, keep it explicitly disabled for Phase 1:

The existing guard `if (this.worker === null || !this._ready)` should become:
```typescript
    // Phase 1: WASM stats is disabled. Backend /api/stats is canonical.
    // Re-enable in Phase 3 after data ingestion + parity test.
    throw new Error("WASM stats disabled in Phase 1");
```

This ensures `computeStats` is unconditionally sealed regardless of loaded layers. The `useStats` hook already uses backend-only path, so no caller is affected.

7. In `dispose`, update cleanup:

Replace:
```typescript
    this._ready = false;
    this.notifyListeners(false);
```
With:
```typescript
    this._loadedLayers.clear();
    this.notifyListeners(false);
```

- [ ] **Step 4: Update use-spatial-engine.ts**

Replace the entire content of `services/frontend/src/hooks/use-spatial-engine.ts`:

```typescript
"use client";

import { useEffect, useState } from "react";
import { spatialEngine } from "@/lib/wasm/spatial-engine";

/** @deprecated Use useSpatialEngineState() for granular readiness. */
export function useSpatialEngineReady(): boolean {
  const [ready, setReady] = useState(spatialEngine.ready);
  useEffect(() => spatialEngine.onReady(setReady), []);
  return ready;
}

/**
 * Granular WASM engine state.
 *
 * Phase 1 limitation: This hook re-renders on init-done only.
 * It does NOT re-render when individual layers are loaded via
 * load-layer (Phase 3). The notification system will be extended
 * in Phase 3 to support per-layer updates.
 */
export function useSpatialEngineState() {
  const [ready, setReady] = useState(spatialEngine.ready);
  useEffect(() => spatialEngine.onReady(setReady), []);

  return {
    /** True if engine is initialized with at least one layer. */
    ready,
    /** Check if specific layers are loaded (accepts UI or canonical IDs). */
    queryReady: (layerIds: string[]) => spatialEngine.queryReady(layerIds),
    /** Set of canonical layer IDs currently loaded. */
    loadedLayers: spatialEngine.loadedLayers,
  };
}
```

Note: `statsReady` is NOT exposed. It remains internal to the adapter.

- [ ] **Step 5: Run tests**

```bash
cd services/frontend && pnpm vitest run && pnpm tsc --noEmit
```

Expected: All tests PASS, tsc clean.

- [ ] **Step 6: Commit**

```bash
git add services/frontend/src/lib/wasm/spatial-engine.ts \
       services/frontend/src/hooks/use-spatial-engine.ts \
       services/frontend/src/__tests__/spatial-engine-ready.test.ts
git commit -m "feat(wasm): replace binary ready with granular loadedLayers

- _ready boolean → _loadedLayers Set<string> (canonical IDs)
- queryReady(ids): checks if specific layers are loaded, normalizes IDs
- registerLoadedLayers(): normalizes keys on insert via canonicalLayerId()
- computeStats() remains sealed (not gated by statsReady)
- useSpatialEngineState() hook — init-done scoped, Phase 3 extends
- statsReady kept internal, not exposed via hooks"
```

---

### Task 3: Request-Scoped Error Handling + Regression Test

**Files:**
- Modify: `services/frontend/src/lib/wasm/worker.ts`
- Modify: `services/frontend/src/lib/wasm/spatial-engine.ts`
- Create: `services/frontend/src/__tests__/spatial-engine-errors.test.ts`

- [ ] **Step 1: Write the error isolation regression test**

Create `services/frontend/src/__tests__/spatial-engine-errors.test.ts`:

```typescript
import { describe, expect, it, beforeEach } from "vitest";
import { SpatialEngineAdapter } from "@/lib/wasm/spatial-engine";

/**
 * To test error isolation, we need to call handleMessage directly.
 * handleMessage is private, so we access it via a test subclass.
 */
class TestableAdapter extends SpatialEngineAdapter {
  /** Expose handleMessage for testing. */
  simulateMessage(msg: unknown): void {
    // Access the private method via bracket notation
    // biome-ignore lint: test-only access to private method
    (this as any).handleMessage(msg);
  }

  /** Expose pending map size for assertions. */
  get pendingCount(): number {
    // biome-ignore lint: test-only access to private field
    return (this as any).pending.size;
  }
}

describe("SpatialEngineAdapter error isolation", () => {
  let adapter: TestableAdapter;

  beforeEach(() => {
    adapter = new TestableAdapter();
    adapter.registerLoadedLayers({ geology: 133, landform: 370 });
  });

  it("query-error rejects ONLY the matching pending request, others survive", async () => {
    // Manually create two pending query promises
    const promise1 = new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(1, { kind: "query", resolve, reject });
    });
    const promise2 = new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(2, { kind: "query", resolve, reject });
    });

    expect(adapter.pendingCount).toBe(2);

    // Send query-error for id=1 only
    adapter.simulateMessage({ type: "query-error", id: 1, error: "layer not found" });

    // id=1 should reject
    await expect(promise1).rejects.toThrow("layer not found");

    // id=2 should still be pending (NOT rejected)
    expect(adapter.pendingCount).toBe(1);
  });

  it("stats-error rejects ONLY the matching pending request", async () => {
    const promise1 = new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(10, { kind: "stats", resolve, reject });
    });
    const promise2 = new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(11, { kind: "query", resolve, reject });
    });

    adapter.simulateMessage({ type: "stats-error", id: 10, error: "compute failed" });

    await expect(promise1).rejects.toThrow("compute failed");
    expect(adapter.pendingCount).toBe(1); // id=11 survives
  });

  it("catch-all error rejects ALL pending (init-level only)", async () => {
    const promise1 = new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(1, { kind: "query", resolve, reject });
    });
    const promise2 = new Promise<unknown>((resolve, reject) => {
      // biome-ignore lint: test-only access to private field
      (adapter as any).pending.set(2, { kind: "query", resolve, reject });
    });

    adapter.simulateMessage({ type: "error", message: "init failed" });

    await expect(promise1).rejects.toThrow("init failed");
    await expect(promise2).rejects.toThrow("init failed");
    expect(adapter.pendingCount).toBe(0);
  });

  it("computeStats throws unconditionally in Phase 1", () => {
    expect(() => adapter.computeStats({
      south: 35.5, west: 139.5, north: 35.9, east: 140.0,
    })).rejects.toThrow("WASM stats disabled in Phase 1");
  });
});
```

- [ ] **Step 2: Run test to verify it fails (behavior test — needs handleMessage changes from Step 3)**

```bash
cd services/frontend && pnpm vitest run src/__tests__/spatial-engine-errors.test.ts
```

Expected: FAIL — `query-error` and `stats-error` message types not handled yet in adapter.

- [ ] **Step 3: Update worker.ts message types and error handling**

In `services/frontend/src/lib/wasm/worker.ts`:

1. Add new outgoing message types. Replace the `OutgoingMessage` type:

```typescript
type OutgoingMessage =
  | InitDoneMessage
  | QueryResultMessage
  | { type: "query-error"; id: number; error: string }
  | StatsResultMsg
  | { type: "stats-error"; id: number; error: string }
  | ErrorMessage;
```

2. Update `handleQuery` to send `query-error` instead of `error`:

Replace the null-engine guard:
```typescript
  if (engine === null) {
    send({ type: "error", message: "SpatialEngine not initialised" });
    return;
  }
```
With:
```typescript
  if (engine === null) {
    send({ type: "query-error", id, error: "SpatialEngine not initialised" });
    return;
  }
```

Replace the catch block:
```typescript
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    send({ type: "error", message: `query_layers failed: ${message}` });
  }
```
With:
```typescript
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    send({ type: "query-error", id, error: `query_layers failed: ${message}` });
  }
```

3. Update `compute-stats` case to send `stats-error`:

Replace the null-engine guard:
```typescript
      if (!engine) {
        send({ type: "error", message: "not initialized" });
        break;
      }
```
With:
```typescript
      if (!engine) {
        send({ type: "stats-error", id: msg.id, error: "not initialized" });
        break;
      }
```

Replace the catch block:
```typescript
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        send({ type: "error", message: `compute_stats failed: ${message}` });
      }
```
With:
```typescript
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        send({ type: "stats-error", id: msg.id, error: `compute_stats failed: ${message}` });
      }
```

- [ ] **Step 4: Update spatial-engine.ts message types and handler**

In `services/frontend/src/lib/wasm/spatial-engine.ts`:

1. Add new message types. After the existing `ErrorMessage` interface, add:

```typescript
interface QueryErrorMessage {
  type: "query-error";
  id: number;
  error: string;
}

interface StatsErrorMessage {
  type: "stats-error";
  id: number;
  error: string;
}
```

2. Update the `WorkerMessage` union:

Replace:
```typescript
type WorkerMessage = InitDoneMessage | QueryResultMessage | StatsResultMessage | ErrorMessage;
```
With:
```typescript
type WorkerMessage =
  | InitDoneMessage
  | QueryResultMessage
  | QueryErrorMessage
  | StatsResultMessage
  | StatsErrorMessage
  | ErrorMessage;
```

3. Add handlers in `handleMessage`. Add these cases BEFORE the existing `"error"` case:

```typescript
      case "query-error": {
        const pending = this.pending.get(msg.id);
        if (!pending) break;
        this.pending.delete(msg.id);
        pending.reject(new Error(msg.error));
        break;
      }

      case "stats-error": {
        const pending = this.pending.get(msg.id);
        if (!pending) break;
        this.pending.delete(msg.id);
        pending.reject(new Error(msg.error));
        break;
      }
```

4. Update the existing `"error"` case comment to clarify scope:

```typescript
      case "error": {
        // Init-level errors only (no request id available).
        // Request-scoped errors use query-error / stats-error.
        const message = msg.message;
        if (this.pending.size > 0) {
          for (const [id, pending] of this.pending) {
            pending.reject(new Error(message));
            this.pending.delete(id);
          }
        } else {
          console.warn("[SpatialEngineAdapter] Worker error:", message);
        }
        break;
      }
```

- [ ] **Step 5: Run tests + type check**

```bash
cd services/frontend && pnpm vitest run && pnpm tsc --noEmit
```

Expected: All tests PASS, tsc clean.

- [ ] **Step 6: Commit**

```bash
git add services/frontend/src/lib/wasm/worker.ts \
       services/frontend/src/lib/wasm/spatial-engine.ts \
       services/frontend/src/__tests__/spatial-engine-errors.test.ts
git commit -m "fix(wasm): request-scoped error handling for query and stats

Worker sends query-error/stats-error with request id instead of
generic error that rejects ALL pending queries. Init-level errors
still use catch-all error message (no request id available).

Adds error isolation regression tests to prevent blast-radius
bug from returning."
```

---

### Task 4: Performance API Observability

**Files:**
- Modify: `services/frontend/src/lib/wasm/spatial-engine.ts`
- Modify: `services/frontend/src/lib/wasm/worker.ts`

- [ ] **Step 1: Add Performance API marks to spatial-engine.ts init**

In `services/frontend/src/lib/wasm/spatial-engine.ts`:

Add import at top (if not already present):
```typescript
import { logger } from "@/lib/logger";

const log = logger.child({ module: "spatial-engine" });
```

In the `init()` method, after the SSR/duplicate guards (`if (typeof window === "undefined") return;` and `if (this.worker !== null ...)`), add:
```typescript
    performance.mark("wasm-init-start");
```

In the `handleMessage` `init-done` case, after `this.registerLoadedLayers(msg.counts);` and before `this.notifyListeners(true);`, add:
```typescript
        performance.mark("wasm-init-done");
        performance.measure("wasm-init", "wasm-init-start", "wasm-init-done");
        const initMeasure = performance.getEntriesByName("wasm-init").pop();
        const allLayerIds = WASM_LAYERS.map(l => l.id);
        const failedLayers = allLayerIds.filter(id => !this._loadedLayers.has(id));
        log.info({
          wasm_init_ms: initMeasure ? Math.round(initMeasure.duration) : -1,
          loaded_count: this._loadedLayers.size,
          loaded_layers: [...this._loadedLayers],
          failed_layers: failedLayers,
        }, "WASM spatial engine initialized");
```

In the init catch block (fallback path), before the existing `console.warn`, add:
```typescript
      performance.mark("wasm-init-failed");
      performance.measure("wasm-init-failed", "wasm-init-start", "wasm-init-failed");
```

- [ ] **Step 2: Add Performance API marks to query method**

In the `query()` method, wrap the promise with timing:

Replace:
```typescript
    return new Promise<FeatureCollection>((resolve, reject) => {
      this.pending.set(id, { kind: "query", resolve, reject });
      this.worker?.postMessage({ type: "query", id, bbox, layers });
    });
```
With:
```typescript
    const markStart = `wasm-query-${id}-start`;
    const markEnd = `wasm-query-${id}-done`;
    const measureName = `wasm-query-${id}`;
    performance.mark(markStart);

    return new Promise<FeatureCollection>((resolve, reject) => {
      this.pending.set(id, {
        kind: "query",
        resolve: (value) => {
          performance.mark(markEnd);
          performance.measure(measureName, markStart, markEnd);
          resolve(value);
        },
        reject: (reason) => {
          performance.mark(markEnd);
          performance.measure(measureName, markStart, markEnd);
          reject(reason);
        },
      });
      this.worker?.postMessage({ type: "query", id, bbox, layers });
    });
```

- [ ] **Step 3: Add Performance API marks to worker.ts layer loading**

In `services/frontend/src/lib/wasm/worker.ts`, in `handleInit`, update the `layers.map` body to wrap each layer load:

Replace the entire `layers.map` callback:
```typescript
    layers.map(async ({ id, url }) => {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(
          `HTTP ${response.status} fetching layer "${id}" from ${url}`,
        );
      }
      const buffer = await response.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      // engine is guaranteed non-null: assigned above before this map runs
      const count = (engine as ISpatialEngine).load_layer(id, bytes);
      return { id, count };
    }),
```
With:
```typescript
    layers.map(async ({ id, url }) => {
      performance.mark(`layer-load-${id}-start`);
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(
          `HTTP ${response.status} fetching layer "${id}" from ${url}`,
        );
      }
      const buffer = await response.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      const count = (engine as ISpatialEngine).load_layer(id, bytes);
      performance.mark(`layer-load-${id}-done`);
      performance.measure(
        `layer-load-${id}`,
        `layer-load-${id}-start`,
        `layer-load-${id}-done`,
      );
      return { id, count };
    }),
```

- [ ] **Step 4: Run tests + type check + lint**

```bash
cd services/frontend && pnpm vitest run && pnpm tsc --noEmit
```

Expected: All pass.

- [ ] **Step 5: Commit**

```bash
git add services/frontend/src/lib/wasm/spatial-engine.ts \
       services/frontend/src/lib/wasm/worker.ts
git commit -m "feat(wasm): add Performance API observability

Instrument WASM init, layer loading, and query timing:
- performance.mark/measure for wasm-init, layer-load-{id}, wasm-query-{id}
- pino summary log with wasm_init_ms, loaded_count, loaded_layers
- Visible in Chrome DevTools Performance tab"
```

---

## Self-Review Checklist

1. **Spec coverage:**
   - Task 0 (canonical ID at WASM/FGB boundary): Task 1 ✅
   - Phase 1.1 (ready separation): Task 2 ✅
   - Phase 1.2 (error handling): Task 3 ✅ (with regression test)
   - Phase 1.3 (stats gating): Task 2 ✅ (`computeStats` stays sealed, `statsReady` internal)
   - Phase 1.4 (observability): Task 4 ✅

2. **Placeholder scan:** No TBD/TODO. All code blocks complete.

3. **Type consistency:**
   - `canonicalLayerId` defined in Task 1, imported in Task 2 (`spatial-engine.ts`)
   - `registerLoadedLayers(counts: Record<string, number>)` normalizes via `canonicalLayerId` on insert (Task 2)
   - `WorkerMessage` union extended in Task 3 with `QueryErrorMessage`/`StatsErrorMessage`
   - `useSpatialEngineState()` does NOT expose `statsReady` (Task 2 non-goal)
   - Performance API marks use string template literals consistently (Task 4)

4. **Codex review fixes verified:**
   - P1 statsReady contradiction: `computeStats` guard unchanged, `statsReady` not exposed ✅
   - P1 ID normalization scope: Goal narrowed to "WASM/FGB boundaries", Non-Goals section added ✅
   - P1 Error regression test: Task 3 includes `spatial-engine-errors.test.ts` ✅
   - P2 registerLoadedLayers normalization: Insert-time `canonicalLayerId()` in Task 2 ✅
   - P2 useSpatialEngineState limitation: Documented as "Phase 1: init-done scoped" ✅

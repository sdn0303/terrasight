import type { FeatureCollection } from "geojson";
import { canonicalLayerId } from "@/lib/layer-ids";
import { logger } from "@/lib/logger";
import { ManifestSchema } from "./manifest-schema";

const log = logger.child({ module: "spatial-engine" });

// ---------------------------------------------------------------------------
// Manifest-driven layer loading
// ---------------------------------------------------------------------------

const DATA_BASE = import.meta.env.VITE_DATA_URL ?? "/data/fgb";

async function loadLayerManifest(
  prefCode: string,
): Promise<Array<{ id: string; url: string }>> {
  const res = await fetch(`${DATA_BASE}/manifest.json`);
  if (!res.ok) {
    throw new Error(`Manifest fetch failed: ${res.status}`);
  }
  const raw: unknown = await res.json();
  const manifest = ManifestSchema.parse(raw);

  const layers: Array<{ id: string; url: string }> = [];

  // Prefecture-specific layers
  const prefLayers = manifest.prefectures[prefCode]?.layers ?? [];
  for (const layer of prefLayers) {
    layers.push({ id: layer.id, url: `${DATA_BASE}/${layer.path}` });
  }

  // National layers (always loaded)
  const nationalLayers = manifest.prefectures["national"]?.layers ?? [];
  for (const layer of nationalLayers) {
    layers.push({ id: layer.id, url: `${DATA_BASE}/${layer.path}` });
  }

  return layers;
}

// ---------------------------------------------------------------------------
// Message protocol (mirrors worker.ts — kept in sync manually)
// ---------------------------------------------------------------------------

interface InitDoneMessage {
  type: "init-done";
  counts: Record<string, number>;
}

interface QueryResultMessage {
  type: "query-result";
  id: number;
  geojson: string;
}

interface StatsResultMessage {
  type: "stats-result";
  id: number;
  stats: string; // JSON string from WASM
}

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

interface ErrorMessage {
  type: "error";
  message: string;
}

interface LoadGeoJsonResultMessage {
  type: "load-geojson-result";
  id: number;
  count: number;
}

interface LoadGeoJsonErrorMessage {
  type: "load-geojson-error";
  id: number;
  error: string;
}

interface TlsResultMessage {
  type: "tls-result";
  id: number;
  result: string; // JSON string from WASM
}

interface TlsErrorMessage {
  type: "tls-error";
  id: number;
  error: string;
}

type WorkerMessage =
  | InitDoneMessage
  | QueryResultMessage
  | QueryErrorMessage
  | StatsResultMessage
  | StatsErrorMessage
  | LoadGeoJsonResultMessage
  | LoadGeoJsonErrorMessage
  | TlsResultMessage
  | TlsErrorMessage
  | ErrorMessage;

// ---------------------------------------------------------------------------
// BBox type
// ---------------------------------------------------------------------------

export interface BBox {
  south: number;
  west: number;
  north: number;
  east: number;
}

// ---------------------------------------------------------------------------
// Pending query bookkeeping
// ---------------------------------------------------------------------------

interface PendingFeatureQuery {
  kind: "query";
  resolve: (value: FeatureCollection) => void;
  reject: (reason: unknown) => void;
}

interface PendingPerLayerQuery {
  kind: "query-per-layer";
  resolve: (value: Map<string, FeatureCollection>) => void;
  reject: (reason: unknown) => void;
}

interface PendingStatsQuery {
  kind: "stats";
  resolve: (value: unknown) => void;
  reject: (reason: unknown) => void;
}

interface PendingLoadGeoJsonQuery {
  kind: "load-geojson";
  resolve: (value: number) => void;
  reject: (reason: unknown) => void;
}

interface PendingTlsQuery {
  kind: "tls";
  resolve: (value: unknown) => void;
  reject: (reason: unknown) => void;
}

type PendingQuery =
  | PendingFeatureQuery
  | PendingPerLayerQuery
  | PendingStatsQuery
  | PendingLoadGeoJsonQuery
  | PendingTlsQuery;

// ---------------------------------------------------------------------------
// Adapter
// ---------------------------------------------------------------------------

export class SpatialEngineAdapter {
  private worker: Worker | null = null;
  private _loadedLayers = new Set<string>();
  private _expectedLayers: string[] = [];
  private readonly pending = new Map<number, PendingQuery>();
  private nextId = 0;
  private readonly listeners: ((ready: boolean) => void)[] = [];

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

  /**
   * Initialise the WASM worker and load all layers for the given prefecture.
   * SSR-safe: no-ops when called outside a browser context.
   * Applies a 30-second timeout; on failure logs a warning and enters
   * fallback mode (worker = null, ready = false).
   */
  async init(prefCode = "13"): Promise<void> {
    if (typeof window === "undefined") return;
    if (this.worker !== null || this._loadedLayers.size > 0) return;

    performance.mark("wasm-init-start");

    try {
      const layers = await loadLayerManifest(prefCode);
      this._expectedLayers = layers.map((l) => l.id);

      this.worker = new Worker(new URL("./worker.ts", import.meta.url), {
        type: "module",
      });

      this.worker.onmessage = (event: MessageEvent<unknown>) => {
        const msg = event.data as WorkerMessage;
        this.handleMessage(msg);
      };

      this.worker.onerror = (event: ErrorEvent) => {
        console.warn("[SpatialEngineAdapter] Worker error:", event.message);
      };

      const initPromise = new Promise<void>((resolve, reject) => {
        // Temporary handler to capture the first "init-done" or "error"
        const originalHandler = this.worker?.onmessage;
        if (!this.worker) {
          reject(new Error("Worker unexpectedly null"));
          return;
        }
        this.worker.onmessage = (event: MessageEvent<unknown>) => {
          const msg = event.data as WorkerMessage;
          if (msg.type === "init-done") {
            // Restore normal handler then resolve
            if (this.worker) this.worker.onmessage = originalHandler ?? null;
            this.handleMessage(msg);
            resolve();
          } else if (msg.type === "error") {
            reject(new Error(msg.message));
          } else {
            // Route any race-condition query results through normal handler
            this.handleMessage(msg);
          }
        };
      });

      const timeoutPromise = new Promise<never>((_, reject) =>
        setTimeout(
          () => reject(new Error("SpatialEngine init timed out after 30s")),
          30_000,
        ),
      );

      this.worker.postMessage({ type: "init", layers });

      await Promise.race([initPromise, timeoutPromise]);
    } catch (err) {
      performance.mark("wasm-init-failed");
      performance.measure(
        "wasm-init-failed",
        "wasm-init-start",
        "wasm-init-failed",
      );
      console.warn(
        "[SpatialEngineAdapter] Falling back to FlatGeobuf mode:",
        err instanceof Error ? err.message : err,
      );
      if (this.worker) {
        this.worker.terminate();
        this.worker = null;
      }
      // _loadedLayers stays empty — callers fall back to full FGB load
      this.notifyListeners(false);
    }
  }

  /**
   * Terminate the current worker and re-initialise for a new prefecture.
   * Clears all loaded layer state before re-init.
   */
  async reloadForPrefecture(prefCode: string): Promise<void> {
    if (this.worker) {
      this.worker.terminate();
      this.worker = null;
    }
    this._loadedLayers.clear();
    this._expectedLayers = [];
    this.notifyListeners(false);

    await this.init(prefCode);
  }

  /**
   * Query one or more layers for a given bounding box.
   * Returns a merged FeatureCollection containing features from all requested
   * layers that intersect the bbox.
   */
  async query(bbox: BBox, layers: string[]): Promise<FeatureCollection> {
    if (this.worker === null || this._loadedLayers.size === 0) {
      return { type: "FeatureCollection", features: [] };
    }

    const id = this.nextId++;
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
  }

  /**
   * Query multiple layers for a given bounding box, returning per-layer results.
   * Unlike `query()` which merges all features, this returns a Map keyed by
   * canonical layer ID, each value a separate FeatureCollection.
   */
  async queryPerLayer(
    bbox: BBox,
    layers: string[],
  ): Promise<Map<string, FeatureCollection>> {
    if (this.worker === null || this._loadedLayers.size === 0) {
      return new Map();
    }

    const id = this.nextId++;
    const markStart = `wasm-query-${id}-start`;
    const markEnd = `wasm-query-${id}-done`;
    const measureName = `wasm-query-${id}`;
    performance.mark(markStart);

    return new Promise<Map<string, FeatureCollection>>((resolve, reject) => {
      this.pending.set(id, {
        kind: "query-per-layer",
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
  }

  /**
   * Compute aggregate stats for a given bounding box using the in-memory
   * WASM index. Returns a plain object matching the StatsResponse shape.
   */
  async computeStats(bbox: BBox): Promise<unknown> {
    if (this.worker === null || this._loadedLayers.size === 0) {
      return null;
    }

    const id = this.nextId++;
    return new Promise<unknown>((resolve, reject) => {
      this.pending.set(id, { kind: "stats", resolve, reject });
      this.worker?.postMessage({
        type: "compute-stats",
        id,
        bbox,
      });
    });
  }

  /**
   * Load a GeoJSON string into the WASM R-tree under the given layer ID.
   * Returns the number of features indexed, or 0 if the worker is not ready.
   */
  async loadGeoJsonLayer(layerId: string, geojson: string): Promise<number> {
    if (this.worker === null) {
      return 0;
    }

    const id = this.nextId++;
    return new Promise<number>((resolve, reject) => {
      this.pending.set(id, { kind: "load-geojson", resolve, reject });
      this.worker?.postMessage({
        type: "load-geojson",
        id,
        layerId,
        geojson,
      });
    });
  }

  /**
   * Compute a Terrasight Location Score (TLS) for the given bounding box
   * using the specified weight preset. Returns a plain object or null when
   * the engine is not ready.
   */
  async computeTls(bbox: BBox, preset = "balance"): Promise<unknown> {
    if (this.worker === null || this._loadedLayers.size === 0) {
      return null;
    }

    const id = this.nextId++;
    return new Promise<unknown>((resolve, reject) => {
      this.pending.set(id, { kind: "tls", resolve, reject });
      this.worker?.postMessage({
        type: "compute-tls",
        id,
        bbox,
        preset,
      });
    });
  }

  /**
   * Subscribe to ready-state changes.
   * Returns an unsubscribe function suitable for use as a useEffect cleanup.
   */
  onReady(listener: (ready: boolean) => void): () => void {
    this.listeners.push(listener);
    return () => {
      const idx = this.listeners.indexOf(listener);
      if (idx !== -1) this.listeners.splice(idx, 1);
    };
  }

  /** Terminate the worker and clear pending queries. */
  dispose(): void {
    this.worker?.terminate();
    this.worker = null;
    for (const { reject } of this.pending.values()) {
      reject(new Error("SpatialEngineAdapter disposed"));
    }
    this.pending.clear();
    this._loadedLayers.clear();
    this.notifyListeners(false);
  }

  // ---------------------------------------------------------------------------
  // Private
  // ---------------------------------------------------------------------------

  private handleMessage(msg: WorkerMessage): void {
    switch (msg.type) {
      case "init-done": {
        this.registerLoadedLayers(msg.counts);
        performance.mark("wasm-init-done");
        performance.measure("wasm-init", "wasm-init-start", "wasm-init-done");
        const initMeasure = performance.getEntriesByName("wasm-init").pop();
        const allLayerIds = this._expectedLayers;
        const failedLayers = allLayerIds.filter(
          (id) => !this._loadedLayers.has(id),
        );
        log.info(
          {
            wasm_init_ms: initMeasure ? Math.round(initMeasure.duration) : -1,
            loaded_count: this._loadedLayers.size,
            loaded_layers: [...this._loadedLayers],
            failed_layers: failedLayers,
          },
          "WASM spatial engine initialized",
        );
        this.notifyListeners(true);
        break;
      }

      case "query-result": {
        const pending = this.pending.get(msg.id);
        if (!pending) break;
        this.pending.delete(msg.id);
        if (pending.kind !== "query" && pending.kind !== "query-per-layer") {
          pending.reject(
            new Error("Unexpected query-result for non-query pending entry"),
          );
          break;
        }
        try {
          // query_layers returns an object keyed by layer id whose values are
          // GeoJSON FeatureCollection objects (single JSON.parse, no inner parse).
          const parsed = JSON.parse(msg.geojson) as Record<string, unknown>;

          if (pending.kind === "query-per-layer") {
            // Per-layer mode: return Map<string, FeatureCollection>
            const map = new Map<string, FeatureCollection>();
            for (const [layerId, fc] of Object.entries(parsed)) {
              if (isFeatureCollection(fc)) {
                map.set(layerId, fc as FeatureCollection);
              }
            }
            pending.resolve(map);
          } else {
            // Merged mode: combine all layer features into single FeatureCollection
            const features: FeatureCollection["features"] = [];
            for (const fc of Object.values(parsed)) {
              if (isFeatureCollection(fc)) {
                features.push(...(fc as FeatureCollection).features);
              }
            }
            pending.resolve({ type: "FeatureCollection", features });
          }
        } catch (err) {
          pending.reject(err);
        }
        break;
      }

      case "stats-result": {
        const pending = this.pending.get(msg.id);
        if (!pending) break;
        this.pending.delete(msg.id);
        if (pending.kind !== "stats") {
          pending.reject(
            new Error("Unexpected stats-result for non-stats pending entry"),
          );
          break;
        }
        try {
          const parsed = JSON.parse(msg.stats) as unknown;
          pending.resolve(parsed);
        } catch (err) {
          pending.reject(err);
        }
        break;
      }

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

      case "load-geojson-result": {
        const pending = this.pending.get(msg.id);
        if (!pending) break;
        this.pending.delete(msg.id);
        if (pending.kind !== "load-geojson") {
          pending.reject(
            new Error(
              "Unexpected load-geojson-result for non-load-geojson pending entry",
            ),
          );
          break;
        }
        pending.resolve(msg.count);
        break;
      }

      case "load-geojson-error": {
        const pending = this.pending.get(msg.id);
        if (!pending) break;
        this.pending.delete(msg.id);
        pending.reject(new Error(msg.error));
        break;
      }

      case "tls-result": {
        const pending = this.pending.get(msg.id);
        if (!pending) break;
        this.pending.delete(msg.id);
        if (pending.kind !== "tls") {
          pending.reject(
            new Error("Unexpected tls-result for non-tls pending entry"),
          );
          break;
        }
        try {
          const parsed = JSON.parse(msg.result) as unknown;
          pending.resolve(parsed);
        } catch (err) {
          pending.reject(err);
        }
        break;
      }

      case "tls-error": {
        const pending = this.pending.get(msg.id);
        if (!pending) break;
        this.pending.delete(msg.id);
        pending.reject(new Error(msg.error));
        break;
      }

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

      default: {
        const _exhaustive: never = msg;
        console.warn(
          "[SpatialEngineAdapter] Unknown worker message",
          _exhaustive,
        );
      }
    }
  }

  private notifyListeners(ready: boolean): void {
    for (const listener of this.listeners) {
      listener(ready);
    }
  }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Type guard for GeoJSON FeatureCollection objects. */
function isFeatureCollection(value: unknown): value is FeatureCollection {
  return (
    value !== null &&
    typeof value === "object" &&
    !Array.isArray(value) &&
    "features" in value &&
    Array.isArray((value as { features: unknown }).features)
  );
}

// ---------------------------------------------------------------------------
// Singleton
// ---------------------------------------------------------------------------

export const spatialEngine = new SpatialEngineAdapter();

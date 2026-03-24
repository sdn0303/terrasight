import type { FeatureCollection } from "geojson";
import { layerUrl } from "@/lib/data-url";

// ---------------------------------------------------------------------------
// Layer manifest — all 11 layers loaded into the R-tree
// ---------------------------------------------------------------------------

const WASM_LAYERS = [
  { id: "admin-boundary", url: layerUrl("13", "admin-boundary") },
  { id: "did", url: layerUrl("13", "did") },
  { id: "flood-history", url: layerUrl("13", "flood-history") },
  { id: "geology", url: layerUrl("13", "geology") },
  { id: "landform", url: layerUrl("13", "landform") },
  { id: "liquefaction", url: layerUrl("13", "liquefaction") },
  { id: "railway", url: layerUrl("13", "railway") },
  { id: "soil", url: layerUrl("13", "soil") },
  { id: "fault", url: layerUrl("national", "fault") },
  { id: "volcano", url: layerUrl("national", "volcano") },
  { id: "seismic", url: layerUrl("national", "seismic") },
] as const;

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

interface ErrorMessage {
  type: "error";
  message: string;
}

type WorkerMessage = InitDoneMessage | QueryResultMessage | StatsResultMessage | ErrorMessage;

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

interface PendingStatsQuery {
  kind: "stats";
  resolve: (value: unknown) => void;
  reject: (reason: unknown) => void;
}

type PendingQuery = PendingFeatureQuery | PendingStatsQuery;

// ---------------------------------------------------------------------------
// Adapter
// ---------------------------------------------------------------------------

export class SpatialEngineAdapter {
  private worker: Worker | null = null;
  private _ready = false;
  private readonly pending = new Map<number, PendingQuery>();
  private nextId = 0;
  private readonly listeners: ((ready: boolean) => void)[] = [];

  get ready(): boolean {
    return this._ready;
  }

  /**
   * Initialise the WASM worker and load all layers.
   * SSR-safe: no-ops when called outside a browser context.
   * Applies a 30-second timeout; on failure logs a warning and enters
   * fallback mode (worker = null, ready = false).
   */
  async init(): Promise<void> {
    if (typeof window === "undefined") return;
    if (this.worker !== null || this._ready) return;

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
      setTimeout(() => reject(new Error("SpatialEngine init timed out after 30s")), 30_000),
    );

    this.worker.postMessage({ type: "init", layers: WASM_LAYERS });

    try {
      await Promise.race([initPromise, timeoutPromise]);
    } catch (err) {
      console.warn(
        "[SpatialEngineAdapter] Falling back to FlatGeobuf mode:",
        err instanceof Error ? err.message : err,
      );
      this.worker.terminate();
      this.worker = null;
      // _ready remains false — callers fall back to full FGB load
    }
  }

  /**
   * Query one or more layers for a given bounding box.
   * Returns a merged FeatureCollection containing features from all requested
   * layers that intersect the bbox.
   */
  async query(bbox: BBox, layers: string[]): Promise<FeatureCollection> {
    if (this.worker === null || !this._ready) {
      return { type: "FeatureCollection", features: [] };
    }

    const id = this.nextId++;

    return new Promise<FeatureCollection>((resolve, reject) => {
      this.pending.set(id, { kind: "query", resolve, reject });
      this.worker?.postMessage({ type: "query", id, bbox, layers });
    });
  }

  /**
   * Compute aggregate stats for a given bounding box using the in-memory
   * WASM index. Returns a plain object matching the StatsResponse shape.
   */
  async computeStats(bbox: BBox): Promise<unknown> {
    if (this.worker === null || !this._ready) {
      throw new Error("SpatialEngineAdapter not ready");
    }

    const id = this.nextId++;

    return new Promise<unknown>((resolve, reject) => {
      this.pending.set(id, { kind: "stats", resolve, reject });
      this.worker?.postMessage({ type: "compute-stats", id, bbox });
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
    this._ready = false;
    this.notifyListeners(false);
  }

  // ---------------------------------------------------------------------------
  // Private
  // ---------------------------------------------------------------------------

  private handleMessage(msg: WorkerMessage): void {
    switch (msg.type) {
      case "init-done": {
        this._ready = true;
        this.notifyListeners(true);
        break;
      }

      case "query-result": {
        const pending = this.pending.get(msg.id);
        if (!pending) break;
        this.pending.delete(msg.id);
        if (pending.kind !== "query") {
          pending.reject(new Error("Unexpected query-result for non-query pending entry"));
          break;
        }
        try {
          const parsed = JSON.parse(msg.geojson) as unknown;
          // query_layers returns an object keyed by layer id whose values are
          // GeoJSON FeatureCollection strings. Merge all features.
          const features: FeatureCollection["features"] = [];
          if (
            parsed !== null &&
            typeof parsed === "object" &&
            !Array.isArray(parsed)
          ) {
            for (const raw of Object.values(parsed as Record<string, unknown>)) {
              if (typeof raw !== "string") continue;
              const fc = JSON.parse(raw) as unknown;
              if (
                fc !== null &&
                typeof fc === "object" &&
                !Array.isArray(fc) &&
                "features" in fc &&
                Array.isArray((fc as { features: unknown }).features)
              ) {
                features.push(
                  ...(fc as FeatureCollection).features,
                );
              }
            }
          }
          pending.resolve({ type: "FeatureCollection", features });
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
          pending.reject(new Error("Unexpected stats-result for non-stats pending entry"));
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

      case "error": {
        // Reject any pending query that may have caused this; otherwise log
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
// Singleton
// ---------------------------------------------------------------------------

export const spatialEngine = new SpatialEngineAdapter();

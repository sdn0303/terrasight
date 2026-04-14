/// <reference lib="webworker" />

import type { ISpatialEngine, IWasmModule } from "./wasm.d.ts";

// ---------------------------------------------------------------------------
// Message protocol types
// ---------------------------------------------------------------------------

interface LayerSpec {
  id: string;
  url: string;
}

interface InitMessage {
  type: "init";
  layers: LayerSpec[];
}

interface QueryMessage {
  type: "query";
  id: number;
  bbox: { south: number; west: number; north: number; east: number };
  layers: string[];
}

interface ComputeStatsMsg {
  type: "compute-stats";
  id: number;
  bbox: { south: number; west: number; north: number; east: number };
}

interface LoadGeoJsonMsg {
  type: "load-geojson";
  id: number;
  layerId: string;
  geojson: string;
}

interface ComputeTlsMsg {
  type: "compute-tls";
  id: number;
  bbox: { south: number; west: number; north: number; east: number };
  preset: string;
}

type IncomingMessage =
  | InitMessage
  | QueryMessage
  | ComputeStatsMsg
  | LoadGeoJsonMsg
  | ComputeTlsMsg;

// ---------------------------------------------------------------------------
// Outgoing message types (main thread receives these)
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

interface StatsResultMsg {
  type: "stats-result";
  id: number;
  stats: string; // JSON string from WASM
}

interface ErrorMessage {
  type: "error";
  message: string;
}

type OutgoingMessage =
  | InitDoneMessage
  | QueryResultMessage
  | { type: "query-error"; id: number; error: string }
  | StatsResultMsg
  | { type: "stats-error"; id: number; error: string }
  | { type: "load-geojson-result"; id: number; count: number }
  | { type: "load-geojson-error"; id: number; error: string }
  | { type: "tls-result"; id: number; result: string }
  | { type: "tls-error"; id: number; error: string }
  | ErrorMessage;

// ---------------------------------------------------------------------------
// Worker state
// ---------------------------------------------------------------------------

let engine: ISpatialEngine | null = null;

// ---------------------------------------------------------------------------
// Helper: typed postMessage
// ---------------------------------------------------------------------------

function send(msg: OutgoingMessage): void {
  postMessage(msg);
}

// ---------------------------------------------------------------------------
// Handle "init"
// ---------------------------------------------------------------------------

async function handleInit(layers: LayerSpec[]): Promise<void> {
  // Dynamic import of the WASM glue served from Next.js public/wasm/.
  // The path is held in a variable so tsc does not attempt static module
  // resolution of an absolute URL (which bundler moduleResolution cannot
  // resolve at compile time). The cast to IWasmModule is safe: the shape is
  // declared in wasm.d.ts and validated against the wasm-bindgen output.
  const wasmGluePath = "/wasm/realestate_wasm.js";
  const wasm = (await import(
    /* webpackIgnore: true */ wasmGluePath
  )) as IWasmModule;

  await wasm.default();
  engine = new wasm.SpatialEngine();

  const results = await Promise.allSettled(
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
      // engine is guaranteed non-null: assigned above before this map runs
      const count = (engine as ISpatialEngine).load_layer(id, bytes);
      performance.mark(`layer-load-${id}-done`);
      performance.measure(
        `layer-load-${id}`,
        `layer-load-${id}-start`,
        `layer-load-${id}-done`,
      );
      return { id, count };
    }),
  );

  const counts: Record<string, number> = {};
  for (const result of results) {
    if (result.status === "fulfilled") {
      counts[result.value.id] = result.value.count;
    } else {
      // Log but do not crash — partial loads are acceptable
      console.warn("[SpatialEngine worker] Layer load failed:", result.reason);
    }
  }

  send({ type: "init-done", counts });
}

// ---------------------------------------------------------------------------
// Handle "query"
// ---------------------------------------------------------------------------

function handleQuery(
  id: number,
  bbox: { south: number; west: number; north: number; east: number },
  layers: string[],
): void {
  if (engine === null) {
    send({ type: "query-error", id, error: "SpatialEngine not initialised" });
    return;
  }

  try {
    const geojson = engine.query_layers(
      layers.join(","),
      bbox.south,
      bbox.west,
      bbox.north,
      bbox.east,
    );
    send({ type: "query-result", id, geojson });
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    send({ type: "query-error", id, error: `query_layers failed: ${message}` });
  }
}

// ---------------------------------------------------------------------------
// Message dispatcher
// ---------------------------------------------------------------------------

self.onmessage = (event: MessageEvent<unknown>) => {
  const msg = event.data as IncomingMessage;

  switch (msg.type) {
    case "init":
      handleInit(msg.layers).catch((err: unknown) => {
        const message = err instanceof Error ? err.message : String(err);
        send({ type: "error", message: `init failed: ${message}` });
      });
      break;

    case "query":
      handleQuery(msg.id, msg.bbox, msg.layers);
      break;

    case "compute-stats": {
      if (!engine) {
        send({ type: "stats-error", id: msg.id, error: "not initialized" });
        break;
      }
      try {
        const stats = engine.compute_stats(
          msg.bbox.south,
          msg.bbox.west,
          msg.bbox.north,
          msg.bbox.east,
        );
        send({ type: "stats-result", id: msg.id, stats });
      } catch (err) {
        const message = err instanceof Error ? err.message : String(err);
        send({
          type: "stats-error",
          id: msg.id,
          error: `compute_stats failed: ${message}`,
        });
      }
      break;
    }

    case "load-geojson": {
      if (!engine) {
        send({
          type: "load-geojson-error",
          id: msg.id,
          error: "not initialized",
        });
        break;
      }
      try {
        const count = engine.load_geojson_layer(msg.layerId, msg.geojson);
        send({ type: "load-geojson-result", id: msg.id, count });
      } catch (err) {
        send({
          type: "load-geojson-error",
          id: msg.id,
          error: err instanceof Error ? err.message : String(err),
        });
      }
      break;
    }

    case "compute-tls": {
      if (!engine) {
        send({ type: "tls-error", id: msg.id, error: "not initialized" });
        break;
      }
      try {
        const result = engine.compute_tls(
          msg.bbox.south,
          msg.bbox.west,
          msg.bbox.north,
          msg.bbox.east,
          msg.preset,
        );
        send({ type: "tls-result", id: msg.id, result });
      } catch (err) {
        send({
          type: "tls-error",
          id: msg.id,
          error: err instanceof Error ? err.message : String(err),
        });
      }
      break;
    }

    default: {
      // Exhaustive check
      const _exhaustive: never = msg;
      console.warn("[SpatialEngine worker] Unknown message type", _exhaustive);
    }
  }
};

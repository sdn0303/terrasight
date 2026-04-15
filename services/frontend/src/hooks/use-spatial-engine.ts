"use client";

import { useEffect, useState } from "react";
import { logger } from "@/lib/logger";
import { spatialEngine } from "@/lib/wasm/spatial-engine";

const log = logger.child({ module: "spatial-engine" });

/**
 * Initialize and dispose the WASM spatial engine.
 * Call once at the app root (e.g. App.tsx).
 */
export function useSpatialEngineInit(): void {
  useEffect(() => {
    spatialEngine.init().catch((err: unknown) => {
      log.error({ err }, "WASM spatial engine failed to initialize");
    });
    return () => spatialEngine.dispose();
  }, []);
}

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

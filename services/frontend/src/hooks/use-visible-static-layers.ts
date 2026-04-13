"use client";

import { useQueries, useQuery } from "@tanstack/react-query";
import { deserialize } from "flatgeobuf/lib/mjs/geojson";
import type { Feature, FeatureCollection } from "geojson";
import { useMemo } from "react";
import { useSpatialEngineReady } from "@/hooks/use-spatial-engine";
import { layerUrl } from "@/lib/data-url";
import { canonicalLayerId } from "@/lib/layer-ids";
import { spatialEngine } from "@/lib/wasm/spatial-engine";
import { useMapStore } from "@/stores/map-store";
import { usePrefectureStore } from "@/stores/prefecture-store";

// ---------------------------------------------------------------------------
// FGB prefix code mapping — canonical layer ID → prefCode for FGB URL
// ---------------------------------------------------------------------------

const NATIONAL_LAYERS = new Set(["fault", "volcano", "seismic"]);

// ---------------------------------------------------------------------------
// FGB fallback fetcher
// ---------------------------------------------------------------------------

async function fetchFgbLayer(
  canonicalId: string,
  prefCode: string,
  signal: AbortSignal,
): Promise<FeatureCollection> {
  const url = layerUrl(prefCode, canonicalId);
  const response = await fetch(url, { signal });
  if (!response.ok) {
    throw new Error(`Failed to fetch ${url}: ${response.status}`);
  }
  const features: Feature[] = [];
  if (response.body) {
    for await (const feature of deserialize(response.body as ReadableStream)) {
      features.push(feature as Feature);
    }
  }
  return { type: "FeatureCollection" as const, features };
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/**
 * Batched static layer query with 2-layer cache.
 *
 * **Layer 2 (WASM viewport cache):** bbox-dependent, 5s stale.
 * Single worker roundtrip for all WASM-ready layers via `queryPerLayer`.
 *
 * **Layer 1 (FGB asset cache):** bbox-independent, infinite TTL.
 * Full FlatGeobuf load per layer when WASM is not ready.
 */
export function useVisibleStaticLayers(
  visibleLayerIds: string[],
): Map<string, FeatureCollection> {
  const wasmReady = useSpatialEngineReady();
  const selectedPrefCode = usePrefectureStore((s) => s.selectedPrefCode);

  const prefCodeForLayer = (canonicalId: string): string =>
    NATIONAL_LAYERS.has(canonicalId) ? "national" : selectedPrefCode;

  // Viewport bbox from map store (same pattern as useStaticLayer)
  const latitude = useMapStore((s) => s.viewState.latitude);
  const longitude = useMapStore((s) => s.viewState.longitude);
  const zoom = useMapStore((s) => s.viewState.zoom);

  const bbox = useMemo(() => {
    const latRange = 180 / 2 ** zoom;
    const lngRange = 360 / 2 ** zoom;
    return {
      south: latitude - latRange / 2,
      west: longitude - lngRange / 2,
      north: latitude + latRange / 2,
      east: longitude + lngRange / 2,
    };
  }, [latitude, longitude, zoom]);

  // Normalize to canonical hyphen-case IDs
  const canonicalIds = useMemo(
    () => visibleLayerIds.map(canonicalLayerId),
    [visibleLayerIds],
  );

  // Classify layers by WASM readiness
  const { wasmLayers, fallbackLayers } = useMemo(() => {
    if (!wasmReady)
      return { wasmLayers: [] as string[], fallbackLayers: canonicalIds };
    const wasm: string[] = [];
    const fallback: string[] = [];
    for (const id of canonicalIds) {
      if (spatialEngine.queryReady([id])) {
        wasm.push(id);
      } else {
        fallback.push(id);
      }
    }
    return { wasmLayers: wasm, fallbackLayers: fallback };
  }, [wasmReady, canonicalIds]);

  // --- Layer 2: WASM batch query (bbox-dependent, short stale) ---
  const sortedWasmKey = useMemo(
    () => [...wasmLayers].sort().join(","),
    [wasmLayers],
  );

  const wasmResult = useQuery({
    queryKey: [
      "static-layers-viewport",
      bbox.south,
      bbox.west,
      bbox.north,
      bbox.east,
      sortedWasmKey,
    ],
    queryFn: () => spatialEngine.queryPerLayer(bbox, wasmLayers),
    enabled: wasmLayers.length > 0,
    staleTime: 5_000,
  });

  // --- Layer 1: FGB fallback (bbox-independent, infinite cache) ---
  const fallbackResults = useQueries({
    queries: fallbackLayers.map((id) => {
      const resolvedPrefCode = prefCodeForLayer(id);
      return {
        queryKey: ["static-layer-fallback", resolvedPrefCode, id],
        queryFn: ({ signal }: { signal: AbortSignal }) =>
          fetchFgbLayer(id, resolvedPrefCode, signal),
        staleTime: Number.POSITIVE_INFINITY,
        gcTime: Number.POSITIVE_INFINITY,
      };
    }),
  });

  // Build merged result map
  return useMemo(() => {
    const map = new Map<string, FeatureCollection>();

    // WASM results
    if (wasmResult.data) {
      for (const [layerId, fc] of wasmResult.data) {
        map.set(layerId, fc);
      }
    }

    // FGB fallback results
    for (let i = 0; i < fallbackLayers.length; i++) {
      const layerId = fallbackLayers[i];
      const data = fallbackResults[i]?.data;
      if (layerId !== undefined && data) {
        map.set(layerId, data);
      }
    }

    return map;
  }, [wasmResult.data, fallbackLayers, fallbackResults]);
}

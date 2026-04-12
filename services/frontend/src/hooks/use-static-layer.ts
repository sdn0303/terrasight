"use client";

import type { Feature, FeatureCollection } from "geojson";
import { useMemo } from "react";
import { useQuery } from "@tanstack/react-query";
import { deserialize } from "flatgeobuf/lib/mjs/geojson";
import { layerUrl } from "@/lib/data-url";
import { spatialEngine } from "@/lib/wasm/spatial-engine";
import { useSpatialEngineReady } from "@/hooks/use-spatial-engine";
import { useMapStore } from "@/stores/map-store";
import { usePrefectureStore } from "@/stores/prefecture-store";

/** @deprecated Use `useVisibleStaticLayers` for batched queries. This hook remains as fallback. */
export function useStaticLayer(layerId: string, enabled: boolean) {
  const selectedPrefCode = usePrefectureStore((s) => s.selectedPrefCode);
  const wasmReady = useSpatialEngineReady();

  // Select primitive viewState values to avoid re-creating a new object
  // reference on every render (which would break useSyncExternalStore's
  // snapshot stability requirement and cause infinite update loops).
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

  // WASM path: viewport-aware R-tree query.
  // queryKey includes bbox coords so TanStack Query re-runs on viewport change.
  const wasmResult = useQuery<FeatureCollection>({
    queryKey: [
      "static-layer-wasm",
      layerId,
      bbox.south,
      bbox.west,
      bbox.north,
      bbox.east,
    ],
    queryFn: () => spatialEngine.query(bbox, [layerId]),
    enabled: enabled && wasmReady,
    staleTime: 5_000,
  });

  // Fallback path: full FlatGeobuf load (no bbox filtering).
  const fallbackResult = useQuery<FeatureCollection>({
    queryKey: ["static-layer-fallback", selectedPrefCode, layerId],
    queryFn: async ({ signal }) => {
      const url = layerUrl(selectedPrefCode, layerId);
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
    enabled: enabled && !wasmReady,
    staleTime: Number.POSITIVE_INFINITY,
    gcTime: Number.POSITIVE_INFINITY,
  });

  return wasmReady ? wasmResult : fallbackResult;
}

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

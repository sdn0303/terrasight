import { useCallback, useRef, useState } from "react";
import { API_BASE } from "@/lib/api";
import type { ActiveLayers } from "@/lib/layers";

interface Bounds {
  south: number;
  west: number;
  north: number;
  east: number;
}

export function useMapData(activeLayers: ActiveLayers) {
  const [layerData, setLayerData] = useState<Record<string, GeoJSON.FeatureCollection>>({});
  const [loading, setLoading] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const fetchForBounds = useCallback(
    (bounds: Bounds) => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
      debounceRef.current = setTimeout(async () => {
        const activeIds = Object.entries(activeLayers)
          .filter(([, v]) => v)
          .map(([k]) => k);
        if (activeIds.length === 0) return;
        setLoading(true);
        try {
          const params = new URLSearchParams({
            south: bounds.south.toString(),
            west: bounds.west.toString(),
            north: bounds.north.toString(),
            east: bounds.east.toString(),
            layers: activeIds.join(","),
          });
          const res = await fetch(`${API_BASE}/api/area-data?${params}`);
          if (res.ok) {
            const data = await res.json();
            setLayerData((prev) => ({ ...prev, ...data }));
          }
        } catch (err) {
          console.error("Failed to fetch area data:", err);
        }
        setLoading(false);
      }, 500);
    },
    [activeLayers],
  );

  return { layerData, loading, fetchForBounds };
}

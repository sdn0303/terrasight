"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
  data?: FeatureCollection;
}

export function FloodHistoryLayer({ visible, data: propData }: Props) {
  const selfFetch = useStaticLayer("13", "flood-history", visible && !propData);
  const data = propData ?? selfFetch.data;
  if (!visible || !data) return null;
  return (
    <Source id="flood_history" type="geojson" data={data}>
      <Layer
        id="flood-history-fill"
        type="fill"
        paint={{
          "fill-color": [
            "step",
            ["get", "year"],
            "#1e3a5f",
            1950,
            "#2563eb",
            1980,
            "#7c3aed",
            2000,
            "#ef4444",
          ] as unknown as string,
          "fill-opacity": 0.35,
        }}
      />
      <Layer
        id="flood-history-outline"
        type="line"
        paint={{
          "line-color": "rgba(255, 255, 255, 0.2)",
          "line-width": 0.5,
        }}
      />
    </Source>
  );
}

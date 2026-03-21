"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

export function FloodHistoryLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source
      id="flood_history"
      type="geojson"
      data="/geojson/flood-history-tokyo.geojson"
    >
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

"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

export function FaultLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source id="fault" type="geojson" data="/geojson/fault-kanto.geojson">
      <Layer
        id="fault-confirmed"
        type="line"
        filter={["==", ["get", "faultType2"], "実在断層"] as unknown as boolean}
        paint={{
          "line-color": "#ef4444",
          "line-width": 2,
        }}
      />
      <Layer
        id="fault-inferred"
        type="line"
        filter={["!=", ["get", "faultType2"], "実在断層"] as unknown as boolean}
        paint={{
          "line-color": "#fbbf24",
          "line-width": 2,
          "line-dasharray": [4, 2],
        }}
      />
    </Source>
  );
}

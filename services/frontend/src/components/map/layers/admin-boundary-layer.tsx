"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

export function AdminBoundaryLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source
      id="admin_boundary"
      type="geojson"
      data="/geojson/admin-boundary-tokyo.geojson"
    >
      <Layer
        id="admin-boundary-fill"
        type="fill"
        paint={{
          "fill-color": "rgba(34, 211, 238, 0.03)",
        }}
      />
      <Layer
        id="admin-boundary-line"
        type="line"
        paint={{
          "line-color": "rgba(255, 255, 255, 0.4)",
          "line-width": 1,
        }}
      />
      <Layer
        id="admin-boundary-label"
        type="symbol"
        layout={{
          "text-field": [
            "coalesce",
            ["get", "wardName"],
            ["get", "cityName"],
          ] as unknown as string,
          "text-size": 11,
        }}
        paint={{
          "text-color": "rgba(255, 255, 255, 0.6)",
          "text-halo-color": "#0a0a0f",
          "text-halo-width": 1,
        }}
      />
    </Source>
  );
}

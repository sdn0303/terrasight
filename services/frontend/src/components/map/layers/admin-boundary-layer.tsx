"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
}

export function AdminBoundaryLayer({ visible }: Props) {
  const { data } = useStaticLayer("13", "admin-boundary", visible);
  if (!visible || !data) return null;
  return (
    <Source
      id="admin_boundary"
      type="geojson"
      data={data}
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

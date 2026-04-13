"use client";

import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

export function BoundaryLayer() {
  const { data } = useStaticLayer("admin-boundary", true);
  if (!data) return null;
  return (
    <Source id="n03-boundary" type="geojson" data={data}>
      <Layer
        id="n03-pref-line"
        type="line"
        paint={{
          "line-color": "#ffffff",
          "line-width": ["interpolate", ["linear"], ["zoom"], 4, 0.8, 10, 2],
          "line-opacity": ["interpolate", ["linear"], ["zoom"], 4, 0.6, 10, 0.8],
        }}
      />
      <Layer
        id="n03-muni-line"
        type="line"
        paint={{
          "line-color": "#ffffff",
          "line-width": 0.5,
          "line-opacity": ["interpolate", ["linear"], ["zoom"], 7, 0, 8, 0.3, 12, 0.6],
        }}
      />
      <Layer
        id="n03-pref-label"
        type="symbol"
        layout={{
          "text-field": ["coalesce", ["get", "prefName"], ""] as unknown as string,
          "text-size": ["interpolate", ["linear"], ["zoom"], 4, 10, 8, 14],
          "text-anchor": "center",
        }}
        paint={{
          "text-color": "#ffffff",
          "text-opacity": ["interpolate", ["linear"], ["zoom"], 4, 0.7, 11, 0],
          "text-halo-color": "#000000",
          "text-halo-width": 1,
        }}
      />
      <Layer
        id="n03-muni-label"
        type="symbol"
        minzoom={8}
        layout={{
          "text-field": [
            "coalesce",
            ["get", "cityName"],
            "",
          ] as unknown as string,
          "text-size": ["interpolate", ["linear"], ["zoom"], 8, 0, 10, 10, 14, 13],
          "text-anchor": "center",
        }}
        paint={{
          "text-color": "#ffffff",
          "text-opacity": ["interpolate", ["linear"], ["zoom"], 8, 0, 10, 0.5, 14, 0.8],
          "text-halo-color": "#000000",
          "text-halo-width": 1,
        }}
      />
    </Source>
  );
}

"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
  data?: FeatureCollection;
}

export function StationLayer({ visible, data: propData }: Props) {
  const selfFetch = useStaticLayer("station", visible && !propData);
  const data = propData ?? selfFetch.data;
  if (!visible || !data) return null;
  return (
    <Source id="station" type="geojson" data={data}>
      <Layer
        id="station-circle"
        type="circle"
        paint={{
          "circle-radius": ["interpolate", ["linear"], ["zoom"], 10, 3, 15, 8],
          "circle-color": "#f472b6",
          "circle-opacity": 0.85,
          "circle-stroke-width": 1,
          "circle-stroke-color": "#0c0c14",
        }}
      />
    </Source>
  );
}

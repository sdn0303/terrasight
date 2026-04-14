"use client";

import type { FeatureCollection } from "geojson";
import type { FilterSpecification } from "mapbox-gl";
import { Layer, Source } from "react-map-gl/mapbox";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
  data?: FeatureCollection;
}

export function FaultLayer({ visible, data: propData }: Props) {
  const selfFetch = useStaticLayer("fault", visible && !propData);
  const data = propData ?? selfFetch.data;
  if (!visible || !data) return null;
  return (
    <Source id="fault" type="geojson" data={data}>
      <Layer
        id="fault-confirmed"
        type="line"
        filter={["==", ["get", "faultType2"], "実在断層"] as FilterSpecification}
        paint={{
          "line-color": "#ef4444",
          "line-width": 2,
        }}
      />
      <Layer
        id="fault-inferred"
        type="line"
        filter={["!=", ["get", "faultType2"], "実在断層"] as FilterSpecification}
        paint={{
          "line-color": "#fbbf24",
          "line-width": 2,
          "line-dasharray": [4, 2],
        }}
      />
    </Source>
  );
}

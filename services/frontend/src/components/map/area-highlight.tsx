"use client";

import { Layer } from "react-map-gl/maplibre";
import { useMapStore } from "@/stores/map-store";

export function AreaHighlight() {
  const selectedArea = useMapStore((s) => s.selectedArea);

  if (!selectedArea) return null;

  const sourceId = selectedArea.level === "prefecture" ? "n03-pref" : "n03-muni";
  const property = selectedArea.level === "prefecture" ? "prefCode" : "code";

  return (
    <>
      <Layer
        id="area-highlight-fill"
        type="fill"
        source={sourceId}
        filter={["==", ["get", property], selectedArea.code]}
        paint={{
          "fill-color": "#22d3ee",
          "fill-opacity": 0.1,
        }}
      />
      <Layer
        id="area-highlight-line"
        type="line"
        source={sourceId}
        filter={["==", ["get", property], selectedArea.code]}
        paint={{
          "line-color": "#22d3ee",
          "line-width": 2,
        }}
      />
    </>
  );
}

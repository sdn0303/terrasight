"use client";

import { Layer, Source } from "react-map-gl/maplibre";

interface Props {
  visible: boolean;
}

export function SchoolDistrictLayer({ visible }: Props) {
  if (!visible) return null;
  return (
    <Source
      id="school-district"
      type="geojson"
      data="/geojson/school-district-tokyo.geojson"
    >
      <Layer
        id="school-district-fill"
        type="fill"
        minzoom={12}
        paint={{
          "fill-color": "#4ade80",
          "fill-opacity": 0.2,
        }}
      />
      <Layer
        id="school-district-outline"
        type="line"
        minzoom={12}
        paint={{
          "line-color": "#4ade80",
          "line-width": 1,
          "line-opacity": 0.6,
        }}
      />
    </Source>
  );
}

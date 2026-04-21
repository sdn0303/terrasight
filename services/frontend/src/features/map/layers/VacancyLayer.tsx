"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/mapbox";

interface VacancyLayerProps {
  data: FeatureCollection;
  visible: boolean;
}

/**
 * 空室率タブ: 市区町村ポリゴンを空き家率%で4段階塗り分け。
 * Ref: DESIGN.md Sec 6.4 — 空室率タブ
 */
export function VacancyLayer({ data, visible }: VacancyLayerProps) {
  return (
    <Source id="vacancy-source" type="geojson" data={data}>
      <Layer
        id="vacancy-fill"
        type="fill"
        layout={{ visibility: visible ? "visible" : "none" }}
        paint={{
          "fill-color": [
            "interpolate",
            ["linear"],
            ["coalesce", ["get", "vacancy_rate_pct"], 0],
            0,
            "#22C55E",
            5,
            "#3B82F6",
            10,
            "#FBBF24",
            15,
            "#EF4444",
          ],
          "fill-opacity": 0.45,
        }}
      />
      <Layer
        id="vacancy-outline"
        type="line"
        layout={{ visibility: visible ? "visible" : "none" }}
        paint={{
          "line-color": "#FFFFFF1A",
          "line-width": 1,
        }}
      />
    </Source>
  );
}

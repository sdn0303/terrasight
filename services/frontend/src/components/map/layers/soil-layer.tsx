"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/mapbox";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
  data?: FeatureCollection;
}

/**
 * Soil classification layer (土壌図).
 *
 * Color mapping reflects ground stability risk for real estate investment:
 * - 黒ボク土 (Andosols/volcanic ash) → stable upland
 * - 灰色低地土/グライ土 → soft lowland, flood-prone
 * - 埋立地 → highest liquefaction risk
 */
const SOIL_COLORS: [string, string][] = [
  ["黒ボク土", "#854d0e"], // dark brown — volcanic ash (Kanto Loam)
  ["褐色森林土", "#65a30d"], // green — forest soil, stable
  ["グライ土", "#0ea5e9"], // sky blue — waterlogged, soft
  ["灰色低地土", "#6366f1"], // indigo — alluvial lowland
  ["泥炭土", "#0d9488"], // teal — peat/wetland
  ["褐色低地土", "#8b5cf6"], // violet — river deposit
  ["赤黄色土", "#ea580c"], // orange — laterite
  ["未熟土", "#a3a3a3"], // gray — immature
  ["ポドゾル", "#c084fc"], // light violet — leached
  ["埋立地", "#e11d48"], // rose — reclaimed land (risk!)
  ["水面", "#06b6d4"], // cyan — water
];

export function SoilLayer({ visible, data: propData }: Props) {
  const selfFetch = useStaticLayer("soil", visible && !propData);
  const data = propData ?? selfFetch.data;
  if (!visible || !data) return null;

  const colorExpr: unknown[] = ["match", ["get", "soilCategory"]];
  for (const [category, color] of SOIL_COLORS) {
    colorExpr.push(category, color);
  }
  colorExpr.push("#71717a"); // fallback (未定義)

  return (
    <Source id="soil" type="geojson" data={data}>
      <Layer
        id="soil-fill"
        type="fill"
        paint={{
          "fill-color": colorExpr as unknown as string,
          "fill-opacity": 0.5,
        }}
      />
      <Layer
        id="soil-outline"
        type="line"
        paint={{
          "line-color": "rgba(255, 255, 255, 0.3)",
          "line-width": 0.5,
        }}
      />
    </Source>
  );
}

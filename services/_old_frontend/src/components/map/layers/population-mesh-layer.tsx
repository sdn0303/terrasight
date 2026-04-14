"use client";

import { useMemo } from "react";
import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
  /** Selected year for population display (2020-2050, step 5) */
  selectedYear: number;
}

/**
 * 250m mesh population projection layer (将来人口メッシュ).
 *
 * Uses colorblind-safe diverging scale (violet → gray → emerald):
 * - Population decrease: violet tones
 * - Stable: gray
 * - Population increase: emerald tones
 *
 * The mesh GeoJSON has properties like pop2020, pop2025, ..., pop2050
 * and changeRate for the overall 2020→2050 trend.
 */
export function PopulationMeshLayer({ visible, selectedYear }: Props) {
  const yearKey = `pop${selectedYear}`;
  const { data } = useStaticLayer("population-mesh", visible);

  const fillColor = useMemo(
    () =>
      [
        "interpolate",
        ["linear"],
        ["get", yearKey],
        0,
        "#7c3aed", // violet — depopulated
        500,
        "#a78bfa", // light violet — declining
        1500,
        "#71717a", // gray — stable
        3000,
        "#6ee7b7", // light emerald — growing
        5000,
        "#10b981", // emerald — high growth
      ] as unknown as string,
    [yearKey],
  );

  if (!visible || !data) return null;

  return (
    <Source
      id="population-mesh"
      type="geojson"
      data={data}
    >
      <Layer
        id="population-mesh-fill"
        type="fill"
        minzoom={13}
        paint={{
          "fill-color": fillColor,
          "fill-opacity": 0.5,
        }}
      />
      <Layer
        id="population-mesh-outline"
        type="line"
        minzoom={13}
        paint={{
          "line-color": "rgba(255, 255, 255, 0.15)",
          "line-width": 0.5,
        }}
      />
    </Source>
  );
}

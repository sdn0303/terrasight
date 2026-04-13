"use client";

import type {
  DataDrivenPropertyValueSpecification,
  FilterSpecification,
} from "maplibre-gl";
import { useCallback, useMemo } from "react";
import type { MapLayerMouseEvent } from "react-map-gl/maplibre";
import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";
import {
  CHOROPLETH_FILL_OPACITY,
  CHOROPLETH_LINE_OPACITY,
  CHOROPLETH_LINE_WIDTH,
  SCORE_PALETTE,
} from "@/lib/map-colors";
import type { DataMode } from "@/stores/data-mode-store";
import { useDataModeStore } from "@/stores/data-mode-store";
import { useMapStore } from "@/stores/map-store";
import { usePrefectureStore } from "@/stores/prefecture-store";

// TODO: Confirm actual GeoJSON property names against real FlatGeobuf data.
//       The fields below are based on the dataset spec; verify with:
//         uv run scripts/tools/pipeline/build_fgb.py --pref 13
//       then inspect the resulting .fgb manifest or GeoJSON output.
//
//       Expected properties:
//         level     — "prefecture" | "municipality" (administrative level)
//         pref_code — "13" (2-digit prefecture code)
//         cityName  — ward/city name for label layer
//         score     — 0.0–1.0 normalised TLS composite score (may be absent)

/** Score property field per DataMode.
 *
 * TODO: Extend this map when per-mode score fields are added to the pipeline.
 *       For now every mode falls back to the generic "score" property.
 */
function scoreFieldForMode(mode: DataMode): string {
  switch (mode) {
    case "tls":
      return "score";
    case "land-price":
      return "score";
    case "yield":
      return "score";
    case "risk":
      return "score";
    case "population":
      return "score";
    case "transactions":
      return "score";
    case "hazard":
      return "score";
  }
}

interface MunicipalityMapLayerProps {
  visible?: boolean;
}

/**
 * Click handler for the "municipality-fill" interactive layer.
 *
 * Usage: add "municipality-fill" to the parent MapView's `interactiveLayerIds`
 * and pass this hook's return value to the `onClick` prop with a layer-id guard.
 *
 * Example in the parent map component:
 *   const handleMunicipalityClick = useMunicipalityLayerClick();
 *   <Map interactiveLayerIds={["municipality-fill"]} onClick={handleMunicipalityClick} />
 */
export function useMunicipalityLayerClick(): (e: MapLayerMouseEvent) => void {
  const selectArea = useMapStore((s) => s.selectArea);

  return useCallback(
    (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0];
      if (!feature?.properties) return;
      // TODO: Verify city_code/cityCode and cityName field names against real data.
      const rawCode = feature.properties.city_code ?? feature.properties.cityCode;
      const rawName = feature.properties.cityName;
      const code = typeof rawCode === "string" ? rawCode : undefined;
      const name = typeof rawName === "string" ? rawName : undefined;
      if (code && name) {
        // bbox is omitted here — zoom adjustment is handled by the map consumer
        // once flyTo for municipalities is implemented.
        // TODO: Compute bbox from feature.geometry when available.
        selectArea({
          code,
          name,
          level: "municipality",
          // Placeholder bbox centred on the click point; replace with geometry bounds.
          bbox: {
            south: e.lngLat.lat - 0.02,
            west: e.lngLat.lng - 0.02,
            north: e.lngLat.lat + 0.02,
            east: e.lngLat.lng + 0.02,
          },
        });
      }
    },
    [selectArea],
  );
}

export function MunicipalityMapLayer({
  visible = true,
}: MunicipalityMapLayerProps) {
  const { data } = useStaticLayer("admin-boundary", visible);
  const dataMode = useDataModeStore((s) => s.mode);
  const selectedPrefCode = usePrefectureStore((s) => s.selectedPrefCode);

  // Build the fill-color MapLibre expression based on the active DataMode.
  // The interpolation maps a 0–1 normalised score to bad→mid→good colours.
  const fillColor =
    useMemo((): DataDrivenPropertyValueSpecification<string> => {
      const field = scoreFieldForMode(dataMode);
      return [
        "interpolate",
        ["linear"],
        ["coalesce", ["get", field], 0.5],
        0.0,
        SCORE_PALETTE.bad,
        0.5,
        SCORE_PALETTE.mid,
        1.0,
        SCORE_PALETTE.good,
      ];
    }, [dataMode]);

  // Filter to municipality-level features for the selected prefecture only.
  // Re-computed when selectedPrefCode changes so MapLibre updates the filter
  // expression without a full Source remount.
  // TODO: Confirm "level", "municipality", and "pref_code" field names against real data.
  const municipalityFilter = useMemo(
    // Cast required: MapLibre expression arrays are not assignable to
    // FilterSpecification without widening — same pattern as prefecture-map-layer.tsx.
    () =>
      [
        "all",
        ["==", ["get", "level"], "municipality"],
        ["==", ["get", "pref_code"], selectedPrefCode],
      ] as unknown as FilterSpecification,
    [selectedPrefCode],
  );

  if (!visible || !data) return null;

  return (
    // NOTE: Click interactivity requires "municipality-fill" to be listed in the
    // parent MapView's interactiveLayerIds. Use useMunicipalityLayerClick() to
    // obtain the handler and wire it to the MapView onClick prop.
    <Source id="municipality-boundaries" type="geojson" data={data}>
      {/* Fill layer — interpolated colour by DataMode score field */}
      <Layer
        id="municipality-fill"
        type="fill"
        filter={municipalityFilter}
        paint={{
          "fill-color": fillColor,
          "fill-opacity": CHOROPLETH_FILL_OPACITY,
        }}
      />

      {/* Boundary lines */}
      <Layer
        id="municipality-line"
        type="line"
        filter={municipalityFilter}
        paint={{
          "line-color": "#ffffff",
          "line-width": CHOROPLETH_LINE_WIDTH,
          "line-opacity": CHOROPLETH_LINE_OPACITY,
        }}
      />

      {/* Municipality name labels */}
      {/* TODO: Confirm "cityName" field name against real data. */}
      <Layer
        id="municipality-label"
        type="symbol"
        filter={municipalityFilter}
        layout={{
          "text-field": ["coalesce", ["get", "cityName"]] as unknown as string,
          "text-size": 11,
          "text-anchor": "center",
        }}
        paint={{
          "text-color": "#ffffff",
          "text-halo-color": "#000000",
          "text-halo-width": 1,
        }}
      />
    </Source>
  );
}

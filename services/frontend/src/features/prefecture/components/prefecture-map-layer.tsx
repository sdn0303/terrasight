import type {
  DataDrivenPropertyValueSpecification,
  FilterSpecification,
} from "mapbox-gl";
import { useCallback, useMemo } from "react";
import type { MapMouseEvent } from "react-map-gl/mapbox";
import { Layer, Source } from "react-map-gl/mapbox";
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
//         prefName  — "東京都" (display name)
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

interface PrefectureMapLayerProps {
  visible?: boolean;
}

// TODO: Evaluate whether to merge this Source with AdminBoundaryLayer to avoid
//       two in-memory GeoJSON copies. Options:
//       1. Accept a pre-fetched `data` prop (same pattern as AdminBoundaryLayer)
//          and hoist the fetch to a shared parent.
//       2. Both layers use useStaticLayer("admin-boundary"), so TanStack Query
//          deduplicates the network request automatically — but two MapLibre
//          Source nodes still hold separate in-memory copies of the GeoJSON.

/**
 * Click handler for the "prefecture-fill" interactive layer.
 *
 * Usage: add "prefecture-fill" to the parent MapView's `interactiveLayerIds`
 * and pass this hook's return value to the `onClick` prop with a layer-id guard.
 *
 * Example in the parent map component:
 *   const handlePrefClick = usePrefectureLayerClick();
 *   <Map interactiveLayerIds={["prefecture-fill"]} onClick={handlePrefClick} />
 */
export function usePrefectureLayerClick(): (e: MapMouseEvent) => void {
  const selectPrefecture = usePrefectureStore((s) => s.selectPrefecture);
  const flyToPrefecture = useMapStore((s) => s.flyToPrefecture);

  return useCallback(
    (e: MapMouseEvent) => {
      const feature = e.features?.[0];
      if (!feature?.properties) return;
      // TODO: Verify pref_code and prefName field names against real data.
      const rawCode = feature.properties.pref_code;
      const rawName = feature.properties.prefName;
      const code = typeof rawCode === "string" ? rawCode : undefined;
      const name = typeof rawName === "string" ? rawName : undefined;
      if (code && name) {
        selectPrefecture(code, name);
        flyToPrefecture(code);
      }
    },
    [selectPrefecture, flyToPrefecture],
  );
}

export function PrefectureMapLayer({
  visible = true,
}: PrefectureMapLayerProps) {
  const { data } = useStaticLayer("admin-boundary", visible);
  const dataMode = useDataModeStore((s) => s.mode);

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

  // Filter to prefecture-level features only.
  // TODO: Confirm "level" field name and "prefecture" value against real data.
  const prefFilter = useMemo(
    // Cast required: MapLibre expression arrays are not assignable to
    // FilterSpecification without widening — same pattern as admin-boundary-layer.tsx.
    () =>
      ["==", ["get", "level"], "prefecture"] as unknown as FilterSpecification,
    [],
  );

  if (!visible || !data) return null;

  return (
    // NOTE: Click interactivity requires "prefecture-fill" to be listed in the
    // parent MapView's interactiveLayerIds. Use usePrefectureLayerClick() to
    // obtain the handler and wire it to the MapView onClick prop.
    <Source id="prefecture-boundaries" type="geojson" data={data}>
      {/* Fill layer — interpolated colour by DataMode score field */}
      <Layer
        id="prefecture-fill"
        type="fill"
        filter={prefFilter}
        paint={{
          "fill-color": fillColor,
          "fill-opacity": CHOROPLETH_FILL_OPACITY,
        }}
      />

      {/* Boundary lines */}
      <Layer
        id="prefecture-line"
        type="line"
        filter={prefFilter}
        paint={{
          "line-color": "#ffffff",
          "line-width": CHOROPLETH_LINE_WIDTH,
          "line-opacity": CHOROPLETH_LINE_OPACITY,
        }}
      />

      {/* Prefecture name labels */}
      {/* TODO: Confirm "cityName" vs "prefName" for label text against real data. */}
      <Layer
        id="prefecture-label"
        type="symbol"
        filter={prefFilter}
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

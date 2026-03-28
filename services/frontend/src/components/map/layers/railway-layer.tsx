"use client";

import type { FeatureCollection } from "geojson";
import { Layer, Source } from "react-map-gl/maplibre";
import { useStaticLayer } from "@/hooks/use-static-layer";

interface Props {
  visible: boolean;
  data?: FeatureCollection;
}

/**
 * Railway lines layer (鉄道路線).
 *
 * Source: MLIT 国土数値情報 N02 鉄道データ
 * Geometry: LineString / MultiLineString
 *
 * Properties:
 *   N02_001 — 路線区分 (service type code)
 *   N02_002 — 運営主体 (operator type code: 1=新幹線, 2=JR, 3=公営, 4=民営, etc.)
 *   N02_003 — 路線名 (line name)
 *   N02_004 — 事業者名 (operator name)
 *   N02_005 — 駅名 (station name, null for non-station segments)
 *
 * Color mapping by operator type (N02_002):
 *   1 → Shinkansen:  gold
 *   2 → JR:          cyan
 *   3 → Municipal:   green
 *   4 → Private:     magenta
 *   other →          zinc
 */
export function RailwayLayer({ visible, data: propData }: Props) {
  const selfFetch = useStaticLayer("13", "railway", visible && !propData);
  const data = propData ?? selfFetch.data;
  if (!visible || !data) return null;
  return (
    <Source id="railway" type="geojson" data={data}>
      <Layer
        id="railway-line"
        type="line"
        paint={{
          "line-color": [
            "match",
            ["to-string", ["get", "N02_002"]],
            "1",
            "#fbbf24",
            "2",
            "#22d3ee",
            "3",
            "#4ade80",
            "4",
            "#e879f9",
            "#a1a1aa",
          ] as unknown as string,
          "line-width": [
            "interpolate",
            ["linear"],
            ["zoom"],
            8,
            1,
            12,
            2,
            16,
            4,
          ],
          "line-opacity": 0.85,
        }}
      />
    </Source>
  );
}

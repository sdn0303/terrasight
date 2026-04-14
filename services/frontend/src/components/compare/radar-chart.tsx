"use client";

import {
  PolarAngleAxis,
  PolarGrid,
  Radar,
  RadarChart,
  ResponsiveContainer,
} from "recharts";
import type { TlsResponse } from "@/lib/schemas";

const AXIS_KEYS = [
  "disaster",
  "terrain",
  "livability",
  "future",
  "price",
] as const;
const AXIS_LABELS = ["Disaster", "Terrain", "Livability", "Future", "Price"];
const COLORS = ["#818cf8", "#f59e0b", "#10b981"]; // indigo, amber, emerald

interface RadarComparisonProps {
  axesList: TlsResponse["axes"][];
  labels?: string[];
}

export function RadarComparison({ axesList, labels }: RadarComparisonProps) {
  const data = AXIS_KEYS.map((key, i) => {
    const point: Record<string, string | number> = {
      axis: AXIS_LABELS[i] ?? key,
    };
    for (let j = 0; j < axesList.length; j++) {
      const axes = axesList[j];
      if (axes) {
        point[`P${j}`] = axes[key].score;
      }
    }
    return point;
  });

  return (
    <div className="px-4 py-2" style={{ height: 200 }}>
      <ResponsiveContainer width="100%" height="100%">
        <RadarChart data={data}>
          <PolarGrid stroke="#334155" />
          <PolarAngleAxis
            dataKey="axis"
            tick={{ fill: "#94a3b8", fontSize: 10 }}
          />
          {axesList.map((_, i) => (
            <Radar
              key={`P${i}`}
              name={labels?.[i] ?? `Point ${String.fromCharCode(65 + i)}`}
              dataKey={`P${i}`}
              stroke={COLORS[i % COLORS.length]}
              fill={COLORS[i % COLORS.length]}
              fillOpacity={0.15}
            />
          ))}
        </RadarChart>
      </ResponsiveContainer>
    </div>
  );
}

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

interface SinglePointRadarProps {
  axes: TlsResponse["axes"];
}

export function SinglePointRadar({ axes }: SinglePointRadarProps) {
  const data = AXIS_KEYS.map((key, i) => ({
    axis: AXIS_LABELS[i],
    score: axes[key].score,
  }));

  return (
    <div className="px-4 py-2" style={{ height: 180 }}>
      <div className="text-[9px] font-semibold tracking-wider uppercase text-ds-text-muted mb-1">
        SCORE PROFILE
      </div>
      <ResponsiveContainer width="100%" height="100%">
        <RadarChart data={data}>
          <PolarGrid stroke="#334155" />
          <PolarAngleAxis
            dataKey="axis"
            tick={{ fill: "#94a3b8", fontSize: 9 }}
          />
          <Radar
            name="Score"
            dataKey="score"
            stroke="#818cf8"
            fill="#818cf8"
            fillOpacity={0.2}
          />
        </RadarChart>
      </ResponsiveContainer>
    </div>
  );
}

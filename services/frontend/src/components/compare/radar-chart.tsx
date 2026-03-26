"use client";

import { PolarAngleAxis, PolarGrid, Radar, RadarChart, ResponsiveContainer } from "recharts";
import type { TlsResponse } from "@/lib/schemas";

const AXIS_KEYS = ["disaster", "terrain", "livability", "future", "price"] as const;
const AXIS_LABELS = ["Disaster", "Terrain", "Livability", "Future", "Price"];

interface RadarComparisonProps {
  axesA: TlsResponse["axes"];
  axesB: TlsResponse["axes"];
}

export function RadarComparison({ axesA, axesB }: RadarComparisonProps) {
  const data = AXIS_KEYS.map((key, i) => ({
    axis: AXIS_LABELS[i],
    A: axesA[key].score,
    B: axesB[key].score,
  }));

  return (
    <div className="px-4 py-2" style={{ height: 200 }}>
      <ResponsiveContainer width="100%" height="100%">
        <RadarChart data={data}>
          <PolarGrid stroke="#404040" />
          <PolarAngleAxis dataKey="axis" tick={{ fill: "#a3a3a3", fontSize: 10 }} />
          <Radar name="A" dataKey="A" stroke="#22d3ee" fill="#22d3ee" fillOpacity={0.15} />
          <Radar name="B" dataKey="B" stroke="#f59e0b" fill="#f59e0b" fillOpacity={0.15} />
        </RadarChart>
      </ResponsiveContainer>
    </div>
  );
}

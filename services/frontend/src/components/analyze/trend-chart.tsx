"use client";

import {
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { useTrend } from "@/features/trend/api/use-trend";

interface TrendChartProps {
  lat: number;
  lng: number;
}

export function TrendChart({ lat, lng }: TrendChartProps) {
  const { data: trend, isLoading } = useTrend(lat, lng);

  if (isLoading) {
    return (
      <div className="h-32 bg-ds-bg-tertiary rounded animate-pulse mx-4" />
    );
  }

  if (!trend || trend.data.length === 0) {
    return (
      <div className="px-4 py-3 text-xs text-ds-text-muted text-center">
        No price trend data available for this location
      </div>
    );
  }

  const chartData = trend.data.map(
    (r: { year: number; price_per_sqm: number }) => ({
      year: r.year,
      price: r.price_per_sqm,
    }),
  );

  return (
    <div className="px-4 py-2">
      <div className="text-[9px] font-semibold tracking-wider uppercase text-ds-text-muted mb-1">
        PRICE TREND
      </div>
      <div style={{ height: 120 }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={chartData}>
            <XAxis dataKey="year" tick={{ fill: "#94a3b8", fontSize: 9 }} />
            <YAxis
              tick={{ fill: "#94a3b8", fontSize: 9 }}
              width={50}
              tickFormatter={(v: number) => `¥${(v / 1000).toFixed(0)}k`}
            />
            <Tooltip />
            <Line
              type="monotone"
              dataKey="price"
              stroke="#818cf8"
              strokeWidth={2}
              dot={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

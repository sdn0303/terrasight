"use client";

import {
  CartesianGrid,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { useTrend } from "@/features/trend/api/use-trend";

interface Props {
  lat: number;
  lng: number;
  featureProperties?: Record<string, unknown>;
}

function Skeleton() {
  return (
    <div className="space-y-3">
      <div className="animate-pulse bg-[var(--panel-hover-bg)] rounded h-4 w-3/4" />
      <div className="animate-pulse bg-[var(--panel-hover-bg)] rounded h-4 w-1/2" />
      <div className="animate-pulse bg-[var(--panel-hover-bg)] rounded h-32 w-full" />
    </div>
  );
}

function formatPrice(value: number): string {
  if (value >= 10_000) {
    return `${(value / 10_000).toFixed(1)}万円/m²`;
  }
  return `${value.toLocaleString()}円/m²`;
}

function formatYoY(cagr: number, direction: "up" | "down"): string {
  const sign = direction === "up" ? "+" : "";
  return `${sign}${(cagr * 100).toFixed(1)}%`;
}

export function LandPriceDetail({ lat, lng }: Props) {
  const { data, isPending, isError } = useTrend(lat, lng);

  if (isPending) return <Skeleton />;

  if (isError || data === undefined) {
    return (
      <p className="text-sm" style={{ color: "var(--panel-text-secondary)" }}>
        データを取得できませんでした
      </p>
    );
  }

  const latest = data.data[data.data.length - 1];
  const latestPrice = latest?.price_per_sqm;

  return (
    <div className="space-y-4">
      {/* Address */}
      <p
        className="text-xs truncate"
        style={{ color: "var(--panel-text-secondary)" }}
      >
        {data.location.address}
        <span className="ml-2">（{data.location.distance_m}m先）</span>
      </p>

      {/* Latest price */}
      <div>
        <p
          className="text-[10px] font-semibold uppercase tracking-wider"
          style={{ color: "var(--panel-text-secondary)" }}
        >
          最新地価
        </p>
        <p
          className="mt-0.5 text-2xl font-extrabold"
          style={{ color: "var(--panel-text-primary)" }}
        >
          {latestPrice !== undefined ? formatPrice(latestPrice) : "—"}
        </p>
        <p
          className="mt-1 text-sm font-semibold"
          style={{
            color: data.direction === "up" ? "#22c55e" : "#ef4444",
          }}
        >
          前年比 {formatYoY(data.cagr, data.direction)}
        </p>
      </div>

      {/* 10-year trend chart */}
      <div>
        <p
          className="mb-2 text-[10px] font-semibold uppercase tracking-wider"
          style={{ color: "var(--panel-text-secondary)" }}
        >
          推移
        </p>
        <ResponsiveContainer width="100%" height={120}>
          <LineChart
            data={data.data}
            margin={{ top: 4, right: 4, bottom: 0, left: 0 }}
          >
            <CartesianGrid strokeDasharray="3 3" stroke="var(--panel-border)" />
            <XAxis
              dataKey="year"
              tick={{ fontSize: 10, fill: "var(--panel-text-secondary)" }}
              tickLine={false}
              axisLine={false}
            />
            <YAxis hide />
            <Tooltip
              contentStyle={{
                background: "var(--panel-bg)",
                border: "1px solid var(--panel-border)",
                borderRadius: 8,
                fontSize: 11,
              }}
              formatter={(value) =>
                typeof value === "number"
                  ? [formatPrice(value), "地価"]
                  : [String(value), "地価"]
              }
            />
            <Line
              type="monotone"
              dataKey="price_per_sqm"
              stroke="#6366f1"
              strokeWidth={2}
              dot={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}

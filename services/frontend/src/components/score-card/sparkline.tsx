"use client";

import {
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import type { TrendResponse } from "@/lib/api/schemas/trend";

interface SparklineProps {
  trend: TrendResponse;
}

export function Sparkline({ trend }: SparklineProps) {
  const color =
    trend.direction === "up" ? "var(--accent-success)" : "var(--accent-danger)";
  const cagrPct = (trend.cagr * 100).toFixed(1);
  const sign = trend.direction === "up" ? "+" : "";

  const firstYear = trend.data[0]?.year;
  const lastYear = trend.data[trend.data.length - 1]?.year;

  return (
    <div
      className="rounded-lg p-3"
      style={{ background: "var(--bg-tertiary)" }}
    >
      <div
        className="text-[9px] tracking-[0.15em] mb-2"
        style={{ color: "var(--text-muted)" }}
      >
        PRICE TREND
      </div>
      <div style={{ width: "100%", height: 40 }}>
        <ResponsiveContainer>
          <LineChart data={trend.data}>
            <XAxis dataKey="year" hide />
            <YAxis hide domain={["dataMin", "dataMax"]} />
            <Tooltip
              contentStyle={{
                background: "var(--bg-secondary)",
                border: "1px solid var(--border-primary)",
                fontSize: 10,
                fontFamily: "var(--font-mono)",
              }}
              formatter={(value) => {
                const price =
                  typeof value === "number"
                    ? `¥${value.toLocaleString()}`
                    : String(value);
                return [price, "per sqm"];
              }}
            />
            <Line
              type="monotone"
              dataKey="price_per_sqm"
              stroke={color}
              strokeWidth={2}
              dot={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
      <div className="flex justify-between mt-1 text-[10px]">
        <span style={{ color: "var(--text-muted)" }}>
          {firstYear !== undefined && lastYear !== undefined
            ? `${firstYear} — ${lastYear}`
            : ""}
        </span>
        <span style={{ color }}>
          CAGR: {sign}
          {cagrPct}%
        </span>
      </div>
    </div>
  );
}

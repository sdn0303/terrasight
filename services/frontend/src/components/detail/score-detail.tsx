"use client";

import {
  PolarAngleAxis,
  PolarGrid,
  Radar,
  RadarChart,
  ResponsiveContainer,
  Tooltip,
} from "recharts";
import { useScore } from "@/features/score/api/use-score";
import type { TlsResponse } from "@/lib/api/schemas/score";

interface Props {
  lat: number;
  lng: number;
  featureProperties?: Record<string, unknown>;
}

function Skeleton() {
  return (
    <div className="space-y-3">
      <div className="animate-pulse bg-[var(--panel-hover-bg)] rounded h-4 w-1/2" />
      <div className="animate-pulse bg-[var(--panel-hover-bg)] rounded h-8 w-1/3" />
      <div className="animate-pulse bg-[var(--panel-hover-bg)] rounded h-40 w-full" />
    </div>
  );
}

const GRADE_COLOR: Record<TlsResponse["tls"]["grade"], string> = {
  S: "#6366f1",
  A: "#22c55e",
  B: "#84cc16",
  C: "#f59e0b",
  D: "#f97316",
  E: "#ef4444",
};

const AXIS_LABELS: Record<string, string> = {
  disaster: "防災",
  terrain: "地形",
  livability: "生活利便",
  future: "将来性",
  price: "価格",
};

type AxisKey = keyof TlsResponse["axes"];

function buildRadarData(axes: TlsResponse["axes"]) {
  const keys = Object.keys(axes) as AxisKey[];
  return keys.map((key) => ({
    axis: AXIS_LABELS[key] ?? key,
    score: axes[key].score,
  }));
}

export function ScoreDetail({ lat, lng }: Props) {
  const { data, isPending, isError } = useScore(lat, lng);

  if (isPending) return <Skeleton />;

  if (isError || data === undefined) {
    return (
      <p className="text-sm" style={{ color: "var(--panel-text-secondary)" }}>
        スコアを取得できませんでした
      </p>
    );
  }

  const radarData = buildRadarData(data.axes);
  const gradeColor = GRADE_COLOR[data.tls.grade];

  return (
    <div className="space-y-4">
      {/* TLS score */}
      <div className="flex items-center gap-3">
        <div
          className="flex h-14 w-14 items-center justify-center rounded-2xl font-extrabold text-white text-2xl flex-shrink-0"
          style={{ backgroundColor: gradeColor }}
        >
          {data.tls.grade}
        </div>
        <div>
          <p
            className="text-3xl font-extrabold"
            style={{ color: "var(--panel-text-primary)" }}
          >
            {data.tls.score.toFixed(1)}
          </p>
          <p
            className="text-xs"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            {data.tls.label}
          </p>
        </div>
      </div>

      {/* Radar chart */}
      <div>
        <p
          className="mb-1 text-[10px] font-semibold uppercase tracking-wider"
          style={{ color: "var(--panel-text-secondary)" }}
        >
          軸スコア
        </p>
        <ResponsiveContainer width="100%" height={160}>
          <RadarChart data={radarData}>
            <PolarGrid stroke="var(--panel-border)" />
            <PolarAngleAxis
              dataKey="axis"
              tick={{ fontSize: 10, fill: "var(--panel-text-secondary)" }}
            />
            <Tooltip
              contentStyle={{
                background: "var(--panel-bg)",
                border: "1px solid var(--panel-border)",
                borderRadius: 8,
                fontSize: 11,
              }}
            />
            <Radar
              dataKey="score"
              stroke="#6366f1"
              fill="#6366f1"
              fillOpacity={0.25}
              strokeWidth={2}
            />
          </RadarChart>
        </ResponsiveContainer>
      </div>

      {/* Axis detail table */}
      <div>
        <p
          className="mb-2 text-[10px] font-semibold uppercase tracking-wider"
          style={{ color: "var(--panel-text-secondary)" }}
        >
          詳細
        </p>
        <table className="w-full">
          <tbody>
            {(Object.keys(data.axes) as AxisKey[]).map((key) => (
              <tr key={key}>
                <td
                  className="py-1.5 pr-3 text-xs"
                  style={{ color: "var(--panel-text-secondary)" }}
                >
                  {AXIS_LABELS[key] ?? key}
                </td>
                <td className="py-1.5">
                  <div className="flex items-center gap-2">
                    <div
                      className="h-1.5 rounded-full flex-1"
                      style={{ background: "var(--panel-border)" }}
                    >
                      <div
                        className="h-full rounded-full"
                        style={{
                          width: `${data.axes[key].score}%`,
                          backgroundColor: gradeColor,
                        }}
                      />
                    </div>
                    <span
                      className="text-xs font-semibold w-8 text-right"
                      style={{ color: "var(--panel-text-primary)" }}
                    >
                      {data.axes[key].score.toFixed(0)}
                    </span>
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Cross analysis */}
      <div
        className="rounded-xl p-3 space-y-1"
        style={{ background: "var(--panel-hover-bg)" }}
      >
        <p
          className="text-[10px] font-semibold uppercase tracking-wider"
          style={{ color: "var(--panel-text-secondary)" }}
        >
          クロス分析
        </p>
        <div className="flex gap-3 flex-wrap">
          <span
            className="text-xs"
            style={{ color: "var(--panel-text-primary)" }}
          >
            価値発見:{" "}
            <strong>
              {(data.cross_analysis.value_discovery * 100).toFixed(0)}
            </strong>
          </span>
          <span
            className="text-xs"
            style={{ color: "var(--panel-text-primary)" }}
          >
            需要シグナル:{" "}
            <strong>
              {(data.cross_analysis.demand_signal * 100).toFixed(0)}
            </strong>
          </span>
          <span
            className="text-xs"
            style={{ color: "var(--panel-text-primary)" }}
          >
            地盤安全:{" "}
            <strong>
              {(data.cross_analysis.ground_safety * 100).toFixed(0)}
            </strong>
          </span>
        </div>
      </div>
    </div>
  );
}

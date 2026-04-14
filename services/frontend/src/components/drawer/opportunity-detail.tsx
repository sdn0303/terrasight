"use client";

import {
  Line,
  LineChart,
  PolarAngleAxis,
  PolarGrid,
  Radar,
  RadarChart,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";
import { useQueryClient } from "@tanstack/react-query";
import type { OpportunitiesResponse } from "@/lib/api/schemas/opportunities";
import { queryKeys } from "@/lib/query-keys";
import { useOpportunityScore } from "./hooks/use-opportunity-score";
import { useOpportunityTrend } from "./hooks/use-opportunity-trend";

interface OpportunityDetailProps {
  id: string;
}

const AXIS_LABELS: Record<string, string> = {
  disaster: "災害",
  terrain: "地形",
  livability: "生活",
  future: "将来",
  price: "価格",
};

const GRADE_COLORS: Record<string, string> = {
  S: "#10b981",
  A: "#22c55e",
  B: "#eab308",
  C: "#f97316",
  D: "#ef4444",
  E: "#991b1b",
};

export function OpportunityDetail({ id }: OpportunityDetailProps) {
  const queryClient = useQueryClient();

  // Read opportunity basic info from any cached opportunities list
  const cachedData = queryClient.getQueriesData<OpportunitiesResponse>({
    queryKey: queryKeys.opportunities.all,
  });

  let opportunity: OpportunitiesResponse["items"][number] | undefined;
  for (const [, data] of cachedData) {
    if (data === undefined) continue;
    const found = data.items.find((item) => String(item.id) === id);
    if (found !== undefined) {
      opportunity = found;
      break;
    }
  }

  const lat = opportunity?.lat ?? null;
  const lng = opportunity?.lng ?? null;

  const { data: score, isLoading: scoreLoading } = useOpportunityScore(
    lat,
    lng,
  );
  const { data: trend, isLoading: trendLoading } = useOpportunityTrend(
    lat,
    lng,
  );

  const radarData =
    score !== undefined
      ? Object.entries(score.axes).map(([key, axis]) => ({
          axis: AXIS_LABELS[key] ?? key,
          score: axis.score,
        }))
      : [];

  const trendData =
    trend !== undefined
      ? trend.data.map((r) => ({ year: r.year, price: r.price_per_sqm }))
      : [];

  return (
    <div
      className="flex flex-col gap-4 p-4 overflow-y-auto h-full"
      style={{ color: "var(--panel-text-primary)" }}
    >
      {/* Address header */}
      {opportunity !== undefined ? (
        <h3 className="text-sm font-bold leading-snug">
          {opportunity.address}
        </h3>
      ) : (
        <div
          className="h-4 w-3/4 animate-pulse rounded"
          style={{ background: "var(--panel-border)" }}
        />
      )}

      {/* TLS Score */}
      {scoreLoading ? (
        <ScoreSkeleton />
      ) : score !== undefined ? (
        <div className="text-center">
          <div
            className="text-4xl font-bold"
            style={{ color: GRADE_COLORS[score.tls.grade] ?? "#64748b" }}
          >
            {Math.round(score.tls.score)}
          </div>
          <div className="flex items-center justify-center gap-2 mt-1">
            <span
              className="text-lg font-bold"
              style={{ color: GRADE_COLORS[score.tls.grade] ?? "#64748b" }}
            >
              {score.tls.grade}
            </span>
            <span
              className="text-xs"
              style={{ color: "var(--panel-text-secondary)" }}
            >
              {score.tls.label}
            </span>
          </div>

          {/* Radar chart */}
          <div style={{ height: 160 }}>
            <ResponsiveContainer width="100%" height="100%">
              <RadarChart data={radarData}>
                <PolarGrid stroke="var(--panel-border)" />
                <PolarAngleAxis
                  dataKey="axis"
                  tick={{ fill: "var(--panel-text-secondary)", fontSize: 10 }}
                />
                <Radar
                  dataKey="score"
                  stroke="#818cf8"
                  fill="#818cf8"
                  fillOpacity={0.3}
                />
              </RadarChart>
            </ResponsiveContainer>
          </div>
        </div>
      ) : null}

      {/* Detail table */}
      {opportunity !== undefined && (
        <table className="w-full text-xs border-collapse">
          <tbody>
            <DetailRow label="用途地域" value={opportunity.zone} />
            <DetailRow
              label="建蔽率"
              value={`${opportunity.building_coverage_ratio}%`}
            />
            <DetailRow
              label="容積率"
              value={`${opportunity.floor_area_ratio}%`}
            />
            <DetailRow
              label="価格/㎡"
              value={`¥${opportunity.price_per_sqm.toLocaleString()}`}
            />
            <DetailRow label="リスク" value={opportunity.risk_level} />
            <DetailRow
              label="トレンド"
              value={`${opportunity.trend_pct >= 0 ? "+" : ""}${opportunity.trend_pct.toFixed(1)}%`}
            />
            {opportunity.station !== null && (
              <DetailRow
                label="最寄駅"
                value={`${opportunity.station.name} (${opportunity.station.distance_m}m)`}
              />
            )}
          </tbody>
        </table>
      )}

      {/* Trend chart */}
      {trendLoading ? (
        <div
          className="h-28 animate-pulse rounded"
          style={{ background: "var(--panel-border)" }}
        />
      ) : trendData.length > 0 ? (
        <div>
          <div
            className="text-[9px] font-semibold tracking-wider uppercase mb-1"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            PRICE TREND
          </div>
          <div style={{ height: 112 }}>
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={trendData}>
                <XAxis
                  dataKey="year"
                  tick={{ fill: "var(--panel-text-secondary)", fontSize: 9 }}
                />
                <YAxis
                  tick={{ fill: "var(--panel-text-secondary)", fontSize: 9 }}
                  width={50}
                  tickFormatter={(v: number) =>
                    `¥${(v / 1000).toFixed(0)}k`
                  }
                />
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
      ) : null}
    </div>
  );
}

function DetailRow({ label, value }: { label: string; value: string }) {
  return (
    <tr style={{ borderBottom: "1px solid var(--panel-border)" }}>
      <td
        className="py-1.5 pr-3 font-medium w-24"
        style={{ color: "var(--panel-text-secondary)" }}
      >
        {label}
      </td>
      <td style={{ color: "var(--panel-text-primary)" }}>{value}</td>
    </tr>
  );
}

function ScoreSkeleton() {
  return (
    <div className="flex flex-col items-center gap-2">
      <div
        className="h-10 w-16 animate-pulse rounded"
        style={{ background: "var(--panel-border)" }}
      />
      <div
        className="h-40 w-full animate-pulse rounded"
        style={{ background: "var(--panel-border)" }}
      />
    </div>
  );
}

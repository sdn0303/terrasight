"use client";

import {
  Bar,
  BarChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";

interface Props {
  lat: number;
  lng: number;
  featureProperties?: Record<string, unknown>;
}

function formatPassengers(count: number): string {
  if (count >= 10_000) {
    return `${(count / 10_000).toFixed(1)}万人`;
  }
  return `${count.toLocaleString()}人`;
}

function asString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function asNumber(value: unknown): number | undefined {
  return typeof value === "number" ? value : undefined;
}

interface YearlyDataEntry {
  year: number;
  count: number;
}

function parseYearlyData(value: unknown): YearlyDataEntry[] {
  if (!Array.isArray(value)) return [];
  const result: YearlyDataEntry[] = [];
  for (const item of value) {
    if (
      item !== null &&
      typeof item === "object" &&
      "year" in item &&
      "count" in item &&
      typeof (item as Record<string, unknown>)["year"] === "number" &&
      typeof (item as Record<string, unknown>)["count"] === "number"
    ) {
      result.push({
        year: (item as Record<string, unknown>)["year"] as number,
        count: (item as Record<string, unknown>)["count"] as number,
      });
    }
  }
  return result;
}

export function StationDetail({ featureProperties }: Props) {
  const stationName = asString(featureProperties?.["station_name"]) ?? "駅名不明";
  const lineName = asString(featureProperties?.["line_name"]);
  const passengerCount = asNumber(featureProperties?.["passenger_count"]);
  const yearlyData = parseYearlyData(featureProperties?.["yearly_data"]);

  return (
    <div className="space-y-4">
      {/* Station name */}
      <div>
        <p
          className="text-xl font-extrabold"
          style={{ color: "var(--panel-text-primary)" }}
        >
          {stationName}
        </p>
        {lineName !== undefined && (
          <p
            className="mt-0.5 text-sm"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            {lineName}
          </p>
        )}
      </div>

      {/* Passenger count */}
      {passengerCount !== undefined && (
        <div>
          <p
            className="text-[10px] font-semibold uppercase tracking-wider"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            1日平均乗降客数
          </p>
          <p
            className="mt-0.5 text-2xl font-extrabold"
            style={{ color: "var(--panel-text-primary)" }}
          >
            {formatPassengers(passengerCount)}
          </p>
        </div>
      )}

      {/* Yearly bar chart */}
      {yearlyData.length > 0 && (
        <div>
          <p
            className="mb-2 text-[10px] font-semibold uppercase tracking-wider"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            年次推移
          </p>
          <ResponsiveContainer width="100%" height={100}>
            <BarChart data={yearlyData} margin={{ top: 4, right: 4, bottom: 0, left: 0 }}>
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
                  ? [formatPassengers(value), "乗降客数"]
                  : [String(value), "乗降客数"]
              }
              />
              <Bar dataKey="count" fill="#6366f1" radius={[3, 3, 0, 0]} />
            </BarChart>
          </ResponsiveContainer>
        </div>
      )}

      {passengerCount === undefined && yearlyData.length === 0 && (
        <p
          className="text-xs"
          style={{ color: "var(--panel-text-secondary)" }}
        >
          地図上の駅をクリックして詳細を確認してください
        </p>
      )}
    </div>
  );
}

"use client";

import type { ColumnDef } from "@tanstack/react-table";
import { RiskPill } from "@/components/ui/risk-pill";
import { ScoreChip } from "@/components/ui/score-chip";
import { StatusPill } from "@/components/ui/status-pill";
import type { Opportunity, OpportunityRiskLevel, Signal } from "@/lib/schemas";

/**
 * TanStack Table column defs for the Opportunities dense table.
 * Keeps cell renderers thin so the composition layer stays readable.
 */
export const propertyColumns: ColumnDef<Opportunity>[] = [
  {
    accessorKey: "address",
    header: "Address",
    cell: ({ row }) => (
      <div>
        <div className="font-bold" style={{ color: "var(--neutral-900)" }}>
          {row.original.address}
        </div>
        <div
          className="mt-0.5 text-[9px]"
          style={{ color: "var(--neutral-400)" }}
        >
          {row.original.building_coverage_ratio}% /{" "}
          {row.original.floor_area_ratio}%
        </div>
      </div>
    ),
    size: 220,
  },
  {
    accessorKey: "zone",
    header: "Zone",
    cell: ({ getValue }) => (
      <span style={{ color: "var(--neutral-500)" }}>{getValue<string>()}</span>
    ),
    size: 70,
  },
  {
    accessorKey: "tls",
    header: "TLS",
    cell: ({ getValue }) => <ScoreChip value={getValue<number>()} />,
    size: 70,
  },
  {
    accessorKey: "risk_level",
    header: "Risk",
    cell: ({ getValue }) => (
      <RiskPill level={getValue<OpportunityRiskLevel>()} />
    ),
    size: 70,
  },
  {
    accessorKey: "trend_pct",
    header: "Trend",
    cell: ({ getValue }) => {
      const v = getValue<number>();
      const arrow = v > 0 ? "▲" : v < 0 ? "▼" : "—";
      const color =
        v > 0 ? "#10b981" : v < 0 ? "#ef4444" : "var(--neutral-500)";
      return (
        <span className="font-bold" style={{ color }}>
          {arrow} {v >= 0 ? "+" : ""}
          {v.toFixed(1)}%
        </span>
      );
    },
    size: 80,
  },
  {
    accessorKey: "station",
    header: "Station",
    cell: ({ row }) => {
      const s = row.original.station;
      if (s === null || s.name === "") {
        return <span style={{ color: "var(--neutral-400)" }}>—</span>;
      }
      return (
        <span style={{ color: "var(--neutral-600)" }}>
          {s.name} {s.distance_m}m
        </span>
      );
    },
    size: 140,
  },
  {
    accessorKey: "price_per_sqm",
    header: "¥/㎡",
    cell: ({ getValue }) => {
      const v = getValue<number>();
      return (
        <span style={{ color: "var(--neutral-900)" }}>
          {v >= 10_000 ? `${Math.round(v / 10_000)}万` : `¥${v}`}
        </span>
      );
    },
    size: 90,
  },
  {
    accessorKey: "signal",
    header: "Signal",
    cell: ({ getValue }) => <StatusPill status={getValue<Signal>()} />,
    size: 80,
  },
];

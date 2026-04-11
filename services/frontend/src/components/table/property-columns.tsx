"use client";

import type { ColumnDef } from "@tanstack/react-table";
import type { MouseEvent } from "react";
import { RiskPill } from "@/components/ui/risk-pill";
import { ScoreChip } from "@/components/ui/score-chip";
import { StatusPill } from "@/components/ui/status-pill";
import type { Opportunity, OpportunityRiskLevel, Signal } from "@/lib/schemas";
import { useUIStore } from "@/stores/ui-store";

/**
 * Multi-select checkbox cell that toggles membership in
 * `ui-store.comparePoints`. Capped at 3 selections (store invariant).
 * Stops click propagation so ticking the box does not also trigger the
 * row's onRowClick (which would open the insight drawer for that row).
 */
function SelectCell({ row }: { row: { original: Opportunity } }) {
  const comparePoints = useUIStore((s) => s.comparePoints);
  const addComparePoint = useUIStore((s) => s.addComparePoint);
  const removeComparePoint = useUIStore((s) => s.removeComparePoint);

  const existingIndex = comparePoints.findIndex(
    (p) => p.lat === row.original.lat && p.lng === row.original.lng,
  );
  const checked = existingIndex >= 0;
  const atCapacity = !checked && comparePoints.length >= 3;

  const toggle = (e: MouseEvent<HTMLButtonElement>) => {
    e.stopPropagation();
    if (checked) {
      removeComparePoint(existingIndex);
    } else if (!atCapacity) {
      addComparePoint({
        lat: row.original.lat,
        lng: row.original.lng,
        address: row.original.address,
      });
    }
  };

  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      aria-label={`Compare ${row.original.address}`}
      onClick={toggle}
      disabled={atCapacity}
      className="h-4 w-4 rounded"
      style={{
        background: checked ? "var(--brand-indigo)" : "var(--neutral-100)",
        border: checked ? "none" : "1px solid var(--neutral-200)",
        color: "#fff",
        fontSize: 10,
        lineHeight: "16px",
        textAlign: "center",
        opacity: atCapacity ? 0.4 : 1,
        cursor: atCapacity ? "not-allowed" : "pointer",
      }}
    >
      {checked ? "✓" : ""}
    </button>
  );
}

/**
 * TanStack Table column defs for the Opportunities dense table.
 * Keeps cell renderers thin so the composition layer stays readable.
 * The leading `select` column drives `ui-store.comparePoints` for the
 * Compare drawer tab (Phase 6).
 */
export const propertyColumns: ColumnDef<Opportunity>[] = [
  {
    id: "select",
    header: "",
    cell: ({ row }) => <SelectCell row={row} />,
    size: 32,
  },
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

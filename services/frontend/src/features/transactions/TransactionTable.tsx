"use client";

import {
  createColumnHelper,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  type SortingState,
  useReactTable,
} from "@tanstack/react-table";
import { useState } from "react";
import type { TransactionDetail } from "@/lib/api/schemas/transaction";
import { useUIStore } from "@/stores/ui-store";

const columnHelper = createColumnHelper<TransactionDetail>();

const columns = [
  columnHelper.accessor("district_name", {
    header: "所在地",
    size: 120,
  }),
  columnHelper.accessor("property_type", {
    header: "種別",
    size: 100,
  }),
  columnHelper.accessor("total_price", {
    header: "取引価格(万)",
    size: 110,
    cell: (info) => {
      const v = info.getValue();
      return v != null
        ? `${Math.round(v / 10000).toLocaleString("ja-JP")}`
        : "—";
    },
  }),
  columnHelper.accessor("price_per_sqm", {
    header: "坪単価",
    size: 90,
    cell: (info) => {
      const v = info.getValue();
      return v != null
        ? `${Math.round((v * 3.306) / 10000).toLocaleString("ja-JP")}万`
        : "—";
    },
  }),
  columnHelper.accessor("area_sqm", {
    header: "面積(m²)",
    size: 80,
    cell: (info) => info.getValue()?.toLocaleString("ja-JP") ?? "—",
  }),
  columnHelper.accessor("nearest_station", {
    header: "最寄駅",
    size: 100,
  }),
  columnHelper.accessor("station_walk_min", {
    header: "徒歩",
    size: 60,
    cell: (info) => {
      const v = info.getValue();
      return v != null ? `${v}分` : "—";
    },
  }),
  columnHelper.accessor("building_year", {
    header: "築年",
    size: 70,
  }),
  columnHelper.accessor("building_structure", {
    header: "構造",
    size: 60,
  }),
  columnHelper.accessor("floor_plan", {
    header: "間取り",
    size: 70,
  }),
];

interface TransactionTableProps {
  data: TransactionDetail[];
  cityName?: string;
  totalCount?: number;
}

/**
 * 取引事例タブ: フローティングテーブル。
 * Ref: DESIGN.md Sec 5.4, UIUX_SPEC.md Sec 4.6
 */
export function TransactionTable({
  data,
  cityName,
  totalCount,
}: TransactionTableProps) {
  const activeTab = useUIStore((s) => s.activeTab);
  const [sorting, setSorting] = useState<SortingState>([]);

  const table = useReactTable({
    data,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });

  if (activeTab !== "transactions") return null;

  return (
    <div
      className="fixed overflow-hidden flex flex-col"
      style={{
        left: "calc(var(--ts-sidebar-width) + var(--ts-gap-panel) * 2)",
        top: "calc(var(--ts-tab-height) + var(--ts-gap-panel) * 2 + 4px)",
        width:
          "calc(100vw - var(--ts-sidebar-width) - var(--ts-gap-panel) * 3 - 220px)",
        maxHeight:
          "calc(100vh - var(--ts-tab-height) - var(--ts-gap-panel) * 3 - 16px)",
        background: "var(--ts-bg-panel)",
        borderRadius: "var(--ts-panel-radius)",
        zIndex: 15,
      }}
    >
      {/* Header */}
      <div
        className="flex items-center justify-between px-4 py-3"
        style={{
          borderBottom: "1px solid var(--ts-border-subtle)",
        }}
      >
        <div className="flex items-center gap-2">
          <span
            className="text-sm font-medium"
            style={{ color: "var(--ts-text-primary)" }}
          >
            {cityName ? `${cityName}の取引事例一覧` : "取引事例一覧"}
          </span>
          {totalCount != null && (
            <span
              className="text-xs px-2 py-0.5 rounded"
              style={{
                background: "var(--ts-bg-tab-active)",
                color: "var(--ts-accent)",
              }}
            >
              {totalCount.toLocaleString("ja-JP")}件
            </span>
          )}
        </div>
      </div>

      {/* Table */}
      <div className="overflow-auto flex-1">
        <table
          className="w-full text-xs"
          style={{ color: "var(--ts-text-secondary)" }}
        >
          <thead>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header) => (
                  <th
                    key={header.id}
                    className="px-3 py-2 text-left font-medium whitespace-nowrap cursor-pointer select-none"
                    style={{
                      color: "var(--ts-text-muted)",
                      borderBottom: "1px solid var(--ts-border-divider)",
                      width: header.getSize(),
                    }}
                    onClick={header.column.getToggleSortingHandler()}
                  >
                    {flexRender(
                      header.column.columnDef.header,
                      header.getContext(),
                    )}
                    {header.column.getIsSorted() === "asc"
                      ? " ↑"
                      : header.column.getIsSorted() === "desc"
                        ? " ↓"
                        : ""}
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody>
            {table.getRowModel().rows.map((row, i) => (
              <tr
                key={row.id}
                className="transition-colors hover:bg-white/5"
                style={{
                  background:
                    i % 2 === 1 ? "var(--ts-bg-table-alt)" : "transparent",
                  height: 40,
                }}
              >
                {row.getVisibleCells().map((cell) => (
                  <td
                    key={cell.id}
                    className="px-3 py-1 whitespace-nowrap font-mono"
                  >
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}

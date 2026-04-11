"use client";

import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  type SortingState,
  useReactTable,
} from "@tanstack/react-table";
import { useState } from "react";
import { GRADIENT } from "@/lib/theme-tokens";

interface DenseTableProps<T> {
  columns: ColumnDef<T, unknown>[];
  data: T[];
  onRowClick?: (row: T) => void;
  selectedId?: string | null;
  getRowId: (row: T) => string;
}

/**
 * Thin TanStack Table v8 wrapper with sticky gradient header, sortable
 * columns, and optional row selection via `selectedId`.
 *
 * Selection renders a brand gradient accent bar on the leftmost cell,
 * keeping the hit-target the full row without modifying semantics.
 */
export function DenseTable<T>({
  columns,
  data,
  onRowClick,
  selectedId,
  getRowId,
}: DenseTableProps<T>) {
  const [sorting, setSorting] = useState<SortingState>([]);
  const table = useReactTable({
    data,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getRowId,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
  });

  return (
    <div className="relative h-full overflow-auto">
      <table className="w-full border-collapse text-[11px]">
        <thead className="sticky top-0 z-10">
          {table.getHeaderGroups().map((headerGroup) => (
            <tr
              key={headerGroup.id}
              style={{ background: GRADIENT.tableHeader }}
            >
              {headerGroup.headers.map((header) => (
                <th
                  key={header.id}
                  className="px-4 py-3 text-left text-[9px] font-bold uppercase"
                  style={{ color: "#e0e7ff", letterSpacing: "0.7px" }}
                >
                  {header.isPlaceholder
                    ? null
                    : flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )}
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.map((row) => {
            const isSelected = selectedId !== null && row.id === selectedId;
            return (
              <tr
                key={row.id}
                onClick={() => onRowClick?.(row.original)}
                className="cursor-pointer transition-colors"
                style={{
                  background: isSelected
                    ? "linear-gradient(90deg, rgba(99,102,241,0.08), rgba(168,85,247,0.04) 50%, transparent)"
                    : "transparent",
                  borderBottom: "1px solid var(--neutral-100)",
                  position: "relative",
                }}
              >
                {row.getVisibleCells().map((cell, idx) => (
                  <td
                    key={cell.id}
                    className="px-4 py-3"
                    style={{
                      color: "var(--neutral-900)",
                      position: "relative",
                    }}
                  >
                    {isSelected && idx === 0 && (
                      <span
                        aria-hidden="true"
                        style={{
                          position: "absolute",
                          left: 0,
                          top: 8,
                          bottom: 8,
                          width: 3,
                          background:
                            "linear-gradient(180deg, #6366f1, #a855f7)",
                          borderRadius: "0 3px 3px 0",
                        }}
                      />
                    )}
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

"use client";

import { useOpportunities } from "@/hooks/use-opportunities";
import { SIDEBAR_COLLAPSED_W, SIDEBAR_EXPANDED_W } from "@/lib/layout";
import { useUIStore } from "@/stores/ui-store";
import type { OpportunityRow } from "./opportunities-columns";
import { opportunityColumns } from "./opportunities-columns";
import { OpportunitiesToolbar } from "./opportunities-toolbar";

const ROW_HEIGHT_PX = 48;

function TableHeader() {
  return (
    <thead>
      <tr
        className="flex shrink-0 border-b text-[11px] font-semibold"
        style={{
          display: "flex",
          borderColor: "var(--panel-border)",
          color: "var(--panel-text-secondary)",
          background: "var(--panel-bg)",
        }}
      >
        {opportunityColumns.map((col) => (
          <th
            key={col.key}
            scope="col"
            className="flex items-center px-3 py-2"
            style={{
              width: col.width === "flex" ? undefined : col.width,
              flex: col.width === "flex" ? 1 : undefined,
              minWidth: col.width === "flex" ? 0 : undefined,
            }}
          >
            {col.label}
          </th>
        ))}
      </tr>
    </thead>
  );
}

interface TableRowProps {
  row: OpportunityRow;
  isActive: boolean;
  onRowClick: (row: OpportunityRow) => void;
}

function TableRow({ row, isActive, onRowClick }: TableRowProps) {
  return (
    <tr
      aria-selected={isActive}
      tabIndex={0}
      className="flex cursor-pointer items-center border-b outline-none transition-colors"
      style={{
        display: "flex",
        height: `${ROW_HEIGHT_PX}px`,
        borderColor: "var(--panel-border)",
        background: isActive ? "var(--panel-active-bg)" : undefined,
      }}
      onClick={() => onRowClick(row)}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          onRowClick(row);
        }
      }}
      onMouseEnter={(e) => {
        if (!isActive) {
          (e.currentTarget as HTMLTableRowElement).style.background =
            "var(--panel-hover-bg)";
        }
      }}
      onMouseLeave={(e) => {
        if (!isActive) {
          (e.currentTarget as HTMLTableRowElement).style.background = "";
        }
      }}
    >
      {opportunityColumns.map((col) => (
        <td
          key={col.key}
          className="flex items-center overflow-hidden px-3"
          style={{
            display: "flex",
            width: col.width === "flex" ? undefined : col.width,
            flex: col.width === "flex" ? 1 : undefined,
            minWidth: col.width === "flex" ? 0 : undefined,
          }}
        >
          {col.render(row)}
        </td>
      ))}
    </tr>
  );
}

export function OpportunitiesTable() {
  const tableOpen = useUIStore((s) => s.tableOpen);
  const sidebarCollapsed = useUIStore((s) => s.sidebarCollapsed);
  const selectedOpportunityId = useUIStore((s) => s.selectedOpportunityId);
  const openOpportunityDrawer = useUIStore((s) => s.openOpportunityDrawer);

  const query = useOpportunities(tableOpen);
  const rows = query.data?.items ?? [];
  const total = query.data?.total ?? 0;

  const leftOffset = sidebarCollapsed
    ? `${SIDEBAR_COLLAPSED_W}px`
    : `${SIDEBAR_EXPANDED_W}px`;

  function handleRowClick(row: OpportunityRow) {
    openOpportunityDrawer(String(row.id));
  }

  return (
    <div
      className="absolute top-0 z-20 flex h-full flex-col"
      style={{
        left: leftOffset,
        width: "calc(65vw)",
        background: "var(--panel-bg)",
        borderRight: "1px solid var(--panel-border)",
        boxShadow: "var(--panel-shadow)",
        transform: tableOpen ? "translateX(0)" : "translateX(-100%)",
        transition: "transform 0.3s ease",
        visibility: tableOpen ? "visible" : "hidden",
      }}
    >
      <OpportunitiesToolbar total={total} />
      <table
        aria-label="物件一覧"
        aria-rowcount={total}
        className="flex flex-1 flex-col overflow-hidden"
        style={{ display: "flex", flexDirection: "column" }}
      >
        <TableHeader />
        <tbody className="flex-1 overflow-y-auto" style={{ display: "block" }}>
          {query.isLoading ? (
            <tr>
              <td>
                <LoadingSkeleton />
              </td>
            </tr>
          ) : query.error !== null ? (
            <tr>
              <td>
                <ErrorState onRetry={() => query.refetch()} />
              </td>
            </tr>
          ) : rows.length === 0 ? (
            <tr>
              <td>
                <EmptyState />
              </td>
            </tr>
          ) : (
            rows.map((row) => (
              <TableRow
                key={row.id}
                row={row}
                isActive={selectedOpportunityId === String(row.id)}
                onRowClick={handleRowClick}
              />
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

function LoadingSkeleton() {
  return (
    <div
      className="space-y-px p-2"
      role="status"
      aria-label="物件データを読み込み中"
    >
      {Array.from({ length: 10 }).map((_, i) => (
        <div
          // biome-ignore lint/suspicious/noArrayIndexKey: static skeleton placeholders
          key={i}
          className="animate-pulse rounded"
          style={{
            height: `${ROW_HEIGHT_PX}px`,
            background: "var(--panel-hover-bg)",
          }}
        />
      ))}
    </div>
  );
}

function EmptyState() {
  return (
    <div
      className="flex h-40 items-center justify-center text-[12px]"
      style={{ color: "var(--panel-text-secondary)" }}
    >
      条件に一致する物件がありません
    </div>
  );
}

function ErrorState({ onRetry }: { onRetry: () => void }) {
  return (
    <div
      className="flex h-40 flex-col items-center justify-center gap-3 text-[12px]"
      style={{ color: "var(--panel-text-secondary)" }}
    >
      <span>読み込みに失敗しました</span>
      <button
        type="button"
        onClick={onRetry}
        className="rounded-md px-3 py-1.5 text-[11px] font-medium"
        style={{
          background: "var(--panel-hover-bg)",
          color: "var(--panel-text-primary)",
        }}
      >
        再試行
      </button>
    </div>
  );
}

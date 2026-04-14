"use client";

import { X, SlidersHorizontal, Search } from "lucide-react";
import { useUIStore } from "@/stores/ui-store";

interface OpportunitiesToolbarProps {
  total: number;
}

export function OpportunitiesToolbar({ total }: OpportunitiesToolbarProps) {
  const closeTable = useUIStore((s) => s.closeTable);

  return (
    <div
      className="flex shrink-0 items-center gap-2 border-b px-3 py-2"
      style={{
        borderColor: "var(--panel-border)",
        background: "var(--panel-bg)",
      }}
    >
      {/* Search input */}
      <div
        className="relative flex flex-1 items-center rounded-md border px-2"
        style={{ borderColor: "var(--panel-border)" }}
      >
        <Search
          size={13}
          style={{ color: "var(--panel-text-secondary)" }}
          aria-hidden="true"
        />
        <input
          type="search"
          placeholder="検索..."
          aria-label="物件を検索"
          className="flex-1 bg-transparent py-1 pl-2 text-[12px] outline-none"
          style={{ color: "var(--panel-text-primary)" }}
        />
      </div>

      {/* Filter button (placeholder) */}
      <button
        type="button"
        aria-label="フィルターを開く"
        className="flex items-center gap-1.5 rounded-md border px-2.5 py-1.5 text-[11px] font-medium"
        style={{
          borderColor: "var(--panel-border)",
          color: "var(--panel-text-secondary)",
          background: "var(--panel-bg)",
        }}
      >
        <SlidersHorizontal size={12} aria-hidden="true" />
        フィルター
      </button>

      {/* Count badge */}
      <span
        className="shrink-0 rounded-full px-2 py-0.5 text-[11px] font-semibold"
        style={{
          background: "var(--panel-hover-bg)",
          color: "var(--panel-text-secondary)",
        }}
        aria-live="polite"
        aria-label={`${total}件の物件`}
      >
        {total.toLocaleString("ja-JP")}件
      </span>

      {/* Close button */}
      <button
        type="button"
        onClick={closeTable}
        aria-label="テーブルを閉じる"
        className="flex items-center justify-center rounded-md p-1.5"
        style={{ color: "var(--panel-text-secondary)" }}
      >
        <X size={14} aria-hidden="true" />
      </button>
    </div>
  );
}

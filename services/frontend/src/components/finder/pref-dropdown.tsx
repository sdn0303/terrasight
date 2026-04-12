"use client";

import { ChevronDown } from "lucide-react";
import { usePrefectureStore } from "@/stores/prefecture-store";

/**
 * Prefecture selector. MVP supports only the current dataset coverage (東京都),
 * so this is a display-only pill that reads from usePrefectureStore.
 * Multi-prefecture support (full dropdown) is deferred until additional
 * prefecture datasets are onboarded.
 */
export function PrefDropdown() {
  const prefName = usePrefectureStore((s) => s.selectedPrefName);

  return (
    <button
      type="button"
      className="flex w-full items-center justify-between rounded-[10px] px-3 py-2.5 text-[11px] font-semibold"
      style={{
        background: "var(--neutral-50)",
        border: "1px solid var(--neutral-100)",
        color: "var(--neutral-900)",
        cursor: "not-allowed",
      }}
      aria-label="Prefecture selector (single prefecture MVP)"
    >
      <span>🇯🇵 {prefName}</span>
      <ChevronDown size={12} style={{ color: "var(--neutral-400)" }} />
    </button>
  );
}

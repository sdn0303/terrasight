"use client";

import { ChevronRight } from "lucide-react";
import { useMapStore } from "@/stores/map-store";

export function BreadcrumbNav() {
  const selectedArea = useMapStore((s) => s.selectedArea);
  const selectArea = useMapStore((s) => s.selectArea);

  return (
    <nav
      className="flex items-center gap-1.5 px-4 py-2 text-[10px] font-medium tracking-wide text-ds-text-muted"
      aria-label="Area breadcrumb"
    >
      <button
        type="button"
        onClick={() => selectArea(null)}
        className={
          selectedArea
            ? "text-ds-accent-primary hover:underline"
            : "text-ds-text-primary"
        }
      >
        Japan
      </button>
      {selectedArea && (
        <>
          <ChevronRight size={12} className="text-ds-text-muted" />
          <span className="text-ds-text-primary">{selectedArea.name}</span>
        </>
      )}
    </nav>
  );
}

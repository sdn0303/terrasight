"use client";

import { useMapStore } from "@/stores/map-store";

export function BreadcrumbNav() {
  const selectedArea = useMapStore((s) => s.selectedArea);
  const selectArea = useMapStore((s) => s.selectArea);

  return (
    <nav
      className="flex items-center gap-1.5 px-4 py-2 text-[10px] font-mono tracking-wide text-neutral-500"
      aria-label="Area breadcrumb"
    >
      <button
        type="button"
        onClick={() => selectArea(null)}
        className={selectedArea ? "text-cyan-400 hover:underline" : "text-neutral-300"}
      >
        Japan
      </button>
      {selectedArea && (
        <>
          <span>&gt;</span>
          <span className="text-neutral-300">{selectedArea.name}</span>
        </>
      )}
    </nav>
  );
}

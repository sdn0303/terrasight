"use client";

import type { ThemeId } from "@/lib/theme-definitions";
import type { PointDetailData } from "@/stores/ui-store";

interface MapPointDetailProps {
  data: PointDetailData;
  activeTab: ThemeId;
}

export function MapPointDetail({ data, activeTab }: MapPointDetailProps) {
  return (
    <div className="p-4 space-y-3">
      <div style={{ color: "var(--panel-text-secondary)" }} className="text-xs">
        {data.lat.toFixed(6)}, {data.lng.toFixed(6)}
      </div>
      <div style={{ color: "var(--panel-text-secondary)" }} className="text-xs">
        Theme: {activeTab}
      </div>
      {/* Full implementation in Phase 3b integration */}
    </div>
  );
}

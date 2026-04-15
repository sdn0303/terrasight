"use client";

import { X } from "lucide-react";
import { useEffect } from "react";
import { MapPointDetail } from "@/components/drawer/map-point-detail";
import { OpportunityDetail } from "@/components/drawer/opportunity-detail";
import { CARD_RADIUS, DRAWER_WIDTH, PAGE_INSET } from "@/lib/layout";
import { useUIStore } from "@/stores/ui-store";

/**
 * Right-side drawer that renders when `tableOpen && rightDrawer !== null`.
 * Slides in from the right edge.
 *
 * Content switches on `rightDrawer.type`:
 *  - "opportunity" → OpportunityDetail
 *  - "map-point"   → MapPointDetail
 */
export function RightDrawer() {
  const tableOpen = useUIStore((s) => s.tableOpen);
  const rightDrawer = useUIStore((s) => s.rightDrawer);
  const closeRightDrawer = useUIStore((s) => s.closeRightDrawer);

  const visible = tableOpen && rightDrawer !== null;

  useEffect(() => {
    if (!visible) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") closeRightDrawer();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [visible, closeRightDrawer]);

  return (
    <aside
      aria-label={
        visible && rightDrawer !== null
          ? rightDrawer.type === "opportunity"
            ? "物件詳細"
            : "地点情報"
          : undefined
      }
      style={{
        position: "absolute",
        top: 0,
        right: 0,
        width: DRAWER_WIDTH,
        height: "100%",
        background: "var(--panel-bg)",
        boxShadow: "-4px 0 20px rgba(0,0,0,0.12)",
        borderRadius: `${CARD_RADIUS.drawer}px 0 0 ${CARD_RADIUS.drawer}px`,
        zIndex: 20,
        display: "flex",
        flexDirection: "column",
        transform: visible ? "translateX(0)" : "translateX(100%)",
        transition: "transform 0.3s ease",
        pointerEvents: visible ? "auto" : "none",
        visibility: visible ? "visible" : "hidden",
      }}
    >
      {visible &&
        rightDrawer !== null &&
        (() => {
          const title =
            rightDrawer.type === "opportunity" ? "物件詳細" : "地点情報";
          return (
            <>
              {/* Header */}
              <header
                className="flex items-center justify-between px-4 py-3 border-b shrink-0"
                style={{
                  borderColor: "var(--panel-border)",
                  paddingTop: PAGE_INSET,
                }}
              >
                <h2
                  className="text-sm font-extrabold"
                  style={{ color: "var(--panel-text-primary)" }}
                >
                  {title}
                </h2>
                <button
                  type="button"
                  onClick={closeRightDrawer}
                  aria-label="Close drawer"
                  className="flex items-center justify-center rounded-[10px]"
                  style={{
                    width: 28,
                    height: 28,
                    background: "var(--panel-border)",
                    color: "var(--panel-text-secondary)",
                  }}
                >
                  <X size={13} aria-hidden="true" />
                </button>
              </header>

              {/* Content */}
              <div className="flex-1 overflow-y-auto">
                {rightDrawer.type === "opportunity" && (
                  <OpportunityDetail id={rightDrawer.id} />
                )}
                {rightDrawer.type === "map-point" && (
                  <MapPointDetail
                    data={rightDrawer.data}
                    activeTab={rightDrawer.activeTab}
                  />
                )}
              </div>
            </>
          );
        })()}
    </aside>
  );
}

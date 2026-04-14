"use client";

import { X } from "lucide-react";
import type { ReactNode } from "react";
import { useEffect } from "react";
import { CARD_RADIUS, DRAWER_WIDTH, PAGE_INSET } from "@/lib/layout";
import { GRADIENT } from "@/lib/theme-tokens";
import type { DrawerTab } from "@/stores/ui-store";

export interface DrawerTabDef {
  id: DrawerTab;
  label: string;
  content: ReactNode;
}

interface InsightDrawerProps {
  open: boolean;
  onClose: () => void;
  title: string;
  subtitle?: string;
  tabs: DrawerTabDef[];
  activeTab: DrawerTab;
  onTabChange: (t: DrawerTab) => void;
}

/**
 * Right-side Layer 2 drawer for selection-based detail display.
 *
 * Renders a header with a SELECTED badge + title + subtitle + close button,
 * a tab bar, and the active tab's content in a scrollable body. Does not
 * own tab state — the caller passes activeTab and onTabChange.
 */
export function InsightDrawer({
  open,
  onClose,
  title,
  subtitle,
  tabs,
  activeTab,
  onTabChange,
}: InsightDrawerProps) {
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  if (!open) return null;

  const active = tabs.find((t) => t.id === activeTab);

  return (
    <aside
      aria-label={title}
      className="absolute flex flex-col overflow-hidden"
      style={{
        top: PAGE_INSET,
        right: PAGE_INSET,
        bottom: PAGE_INSET,
        width: DRAWER_WIDTH,
        background: "var(--card-fill)",
        borderRadius: CARD_RADIUS.drawer,
        boxShadow: "var(--shadow-card-strong)",
        backdropFilter: "blur(24px)",
        zIndex: 25,
      }}
    >
      {/* Header */}
      <header
        className="border-b px-[18px] pb-3 pt-4"
        style={{ borderColor: "var(--neutral-100)" }}
      >
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <span
              className="mb-1.5 inline-flex items-center rounded-full px-2.5 py-[3px] text-[9px] font-extrabold uppercase tracking-wider"
              style={{
                background: GRADIENT.brandTint,
                color: "var(--brand-indigo)",
                letterSpacing: "0.6px",
              }}
            >
              ● SELECTED
            </span>
            <h2
              className="text-[15px] font-extrabold"
              style={{ color: "var(--neutral-900)" }}
            >
              {title}
            </h2>
            {subtitle !== undefined && (
              <p
                className="mt-0.5 text-[10px]"
                style={{ color: "var(--neutral-400)" }}
              >
                {subtitle}
              </p>
            )}
          </div>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close drawer"
            className="flex h-6 w-6 items-center justify-center rounded-[9px]"
            style={{
              background: "var(--neutral-100)",
              color: "var(--neutral-400)",
            }}
          >
            <X size={13} aria-hidden="true" />
          </button>
        </div>

        {/* Tab bar */}
        <div
          role="tablist"
          className="mt-3 flex gap-0.5 rounded-[10px] p-[3px]"
          style={{ background: "var(--neutral-100)" }}
        >
          {tabs.map((tab) => {
            const isActive = tab.id === activeTab;
            return (
              <button
                key={tab.id}
                id={`tab-${tab.id}`}
                type="button"
                role="tab"
                aria-selected={isActive}
                onClick={() => onTabChange(tab.id)}
                className="flex-1 rounded-lg py-1.5 text-center text-[9px]"
                style={{
                  background: isActive
                    ? "var(--card-fill-solid)"
                    : "transparent",
                  color: isActive ? "var(--neutral-900)" : "var(--neutral-500)",
                  fontWeight: isActive ? 800 : 600,
                  boxShadow: isActive
                    ? "0 2px 6px rgba(0,0,0,0.08)"
                    : undefined,
                }}
              >
                {tab.label}
              </button>
            );
          })}
        </div>
      </header>

      {/* Body — active tab content */}
      <div
        role="tabpanel"
        aria-labelledby={`tab-${activeTab}`}
        className="flex-1 overflow-y-auto px-[18px] py-3.5"
      >
        {active?.content}
      </div>
    </aside>
  );
}

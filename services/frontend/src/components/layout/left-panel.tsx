"use client";

import { X } from "lucide-react";
import type { ReactNode } from "react";
import { useEffect } from "react";
import {
  CARD_RADIUS,
  GAP,
  LEFT_PANEL_WIDTH,
  PAGE_INSET,
  RAIL_WIDTH,
} from "@/lib/layout";
import { GRADIENT } from "@/lib/theme-tokens";

interface LeftPanelProps {
  open: boolean;
  onClose: () => void;
  title: string;
  subtitle?: string;
  badge?: string;
  footer?: ReactNode;
  children: ReactNode;
}

/**
 * Generic left-side Layer 2 panel used by Finder, Layers, and Themes.
 *
 * Floats on top of the map canvas at a fixed left offset (rail + gap) with
 * header, scrollable body, and optional sticky footer.
 */
export function LeftPanel({
  open,
  onClose,
  title,
  subtitle,
  badge,
  footer,
  children,
}: LeftPanelProps) {
  useEffect(() => {
    if (!open) return;
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <section
      aria-label={title}
      className="absolute flex flex-col overflow-hidden"
      style={{
        top: PAGE_INSET,
        left: PAGE_INSET + RAIL_WIDTH + GAP,
        bottom: PAGE_INSET,
        width: LEFT_PANEL_WIDTH,
        background: "var(--card-fill)",
        borderRadius: CARD_RADIUS.mainPanel,
        boxShadow: "var(--shadow-card-strong)",
        backdropFilter: "blur(24px)",
        zIndex: 25,
      }}
    >
      {/* Header */}
      <header
        className="flex items-start justify-between border-b px-5 py-[18px]"
        style={{ borderColor: "var(--neutral-100)" }}
      >
        <div>
          {badge !== undefined && (
            <span
              className="mb-1.5 inline-flex items-center rounded-full px-2.5 py-[3px] text-[9px] font-extrabold uppercase tracking-wider"
              style={{
                background: GRADIENT.brandTint,
                color: "var(--brand-indigo)",
                letterSpacing: "0.6px",
              }}
            >
              ● {badge}
            </span>
          )}
          <h2
            className="text-base font-extrabold"
            style={{ color: "var(--neutral-900)", letterSpacing: "-0.2px" }}
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
          aria-label="Close panel"
          className="flex h-7 w-7 items-center justify-center rounded-[10px]"
          style={{
            background: "var(--neutral-100)",
            color: "var(--neutral-400)",
          }}
        >
          <X size={14} aria-hidden="true" />
        </button>
      </header>

      {/* Body */}
      <div className="flex-1 overflow-y-auto px-5 py-4">{children}</div>

      {/* Footer */}
      {footer !== undefined && (
        <div
          className="border-t px-5 py-3"
          style={{
            borderColor: "var(--neutral-100)",
            background: "var(--card-fill-solid)",
          }}
        >
          {footer}
        </div>
      )}
    </section>
  );
}

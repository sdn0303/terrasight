"use client";

import { X } from "lucide-react";
import type { KeyboardEvent, ReactNode } from "react";
import {
  BOTTOM_SHEET_MAX_PCT,
  BOTTOM_SHEET_MIN_PCT,
  CARD_RADIUS,
  MAIN_LEFT_BASE,
  PAGE_INSET,
} from "@/lib/layout";

interface BottomSheetProps {
  open: boolean;
  onClose: () => void;
  title: string;
  subtitle?: string;
  heightPct: number;
  onHeightChange: (pct: number) => void;
  actions?: ReactNode;
  children: ReactNode;
}

const STEP_PCT = 5;

/**
 * Bottom-docked Layer 2 sheet with a keyboard-accessible drag handle.
 *
 * In Phase 1 the resize handle supports keyboard control only
 * (ArrowUp/ArrowDown ±5%). Pointer drag can be added in a later phase
 * without changing the prop contract.
 */
export function BottomSheet({
  open,
  onClose,
  title,
  subtitle,
  heightPct,
  onHeightChange,
  actions,
  children,
}: BottomSheetProps) {
  if (!open) return null;

  const clampedPct = Math.max(
    BOTTOM_SHEET_MIN_PCT,
    Math.min(BOTTOM_SHEET_MAX_PCT, heightPct),
  );

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "ArrowUp") {
      e.preventDefault();
      onHeightChange(Math.min(BOTTOM_SHEET_MAX_PCT, clampedPct + STEP_PCT));
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      onHeightChange(Math.max(BOTTOM_SHEET_MIN_PCT, clampedPct - STEP_PCT));
    }
  };

  return (
    <section
      aria-label={title}
      className="absolute flex flex-col overflow-hidden"
      style={{
        left: MAIN_LEFT_BASE,
        right: PAGE_INSET,
        bottom: PAGE_INSET,
        height: `${clampedPct}%`,
        background: "var(--card-fill)",
        borderRadius: CARD_RADIUS.bottomSheet,
        boxShadow: "var(--shadow-card-strong)",
        backdropFilter: "blur(24px)",
        zIndex: 20,
      }}
    >
      <div
        role="slider"
        aria-label="Resize bottom sheet"
        aria-valuemin={BOTTOM_SHEET_MIN_PCT}
        aria-valuemax={BOTTOM_SHEET_MAX_PCT}
        aria-valuenow={clampedPct}
        tabIndex={0}
        onKeyDown={handleKeyDown}
        className="flex h-6 cursor-row-resize items-center justify-center"
      >
        <div
          aria-hidden="true"
          style={{
            width: 36,
            height: 4,
            background: "var(--neutral-200)",
            borderRadius: 999,
          }}
        />
      </div>

      <header
        className="flex items-center justify-between border-b px-6 pb-3"
        style={{ borderColor: "var(--neutral-100)" }}
      >
        <div>
          <h2
            className="text-base font-extrabold"
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
        <div className="flex items-center gap-2">
          {actions}
          <button
            type="button"
            onClick={onClose}
            aria-label="Close bottom sheet"
            className="flex h-7 w-7 items-center justify-center rounded-[10px]"
            style={{
              background: "var(--neutral-100)",
              color: "var(--neutral-400)",
            }}
          >
            <X size={14} aria-hidden="true" />
          </button>
        </div>
      </header>

      <div className="flex-1 overflow-auto">{children}</div>
    </section>
  );
}

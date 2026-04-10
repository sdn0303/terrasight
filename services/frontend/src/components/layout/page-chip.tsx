"use client";

import type { CSSProperties, ReactNode } from "react";
import { CARD_RADIUS, PAGE_INSET } from "@/lib/layout";

export type ChipAnchor =
  | "bottom-center"
  | "bottom-right"
  | "bottom-left"
  | "top-right"
  | "top-left";

type ChipOffset = Partial<Record<"top" | "right" | "bottom" | "left", number>>;

interface PageChipProps {
  anchor: ChipAnchor;
  offset?: ChipOffset;
  "aria-label"?: string;
  children: ReactNode;
}

/**
 * Generic floating chip container used by Layer 1/3 contextual overlays
 * (time machine, status chip, stat overlay, zoom controls).
 */
export function PageChip({
  anchor,
  offset,
  "aria-label": ariaLabel,
  children,
}: PageChipProps) {
  const style: CSSProperties = {
    position: "absolute",
    background: "rgba(255, 255, 255, 0.95)",
    borderRadius: CARD_RADIUS.smallChip,
    boxShadow: "var(--shadow-card-subtle)",
    backdropFilter: "blur(16px)",
    padding: "10px 14px",
    zIndex: 15,
  };

  switch (anchor) {
    case "bottom-center":
      style.bottom = offset?.bottom ?? PAGE_INSET;
      style.left = "50%";
      style.transform = "translateX(-50%)";
      break;
    case "bottom-right":
      style.bottom = offset?.bottom ?? PAGE_INSET;
      style.right = offset?.right ?? PAGE_INSET;
      break;
    case "bottom-left":
      style.bottom = offset?.bottom ?? PAGE_INSET;
      style.left = offset?.left ?? PAGE_INSET;
      break;
    case "top-right":
      style.top = offset?.top ?? PAGE_INSET;
      style.right = offset?.right ?? PAGE_INSET;
      break;
    case "top-left":
      style.top = offset?.top ?? PAGE_INSET;
      style.left = offset?.left ?? PAGE_INSET;
      break;
  }

  return (
    // biome-ignore lint/a11y/useAriaPropsSupportedByRole: generic chip container; aria-label labels the region for screen readers without imposing a specific ARIA role
    <div style={style} aria-label={ariaLabel}>
      {children}
    </div>
  );
}

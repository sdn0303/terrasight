

import type { ReactNode } from "react";

interface MapCanvasFrameProps {
  children: ReactNode;
  "aria-label"?: string;
}

/**
 * Root wrapper for the single-page map-canvas shell.
 *
 * Renders a full-viewport relative container. The live MapLibre canvas and
 * all overlay elements (rail, panels, drawer, bottom sheet, chips) position
 * themselves inside via absolute placement. There is no page background —
 * the map itself is the canvas.
 */
export function MapCanvasFrame({
  children,
  "aria-label": ariaLabel = "Map canvas",
}: MapCanvasFrameProps) {
  return (
    <section
      className="relative h-screen w-screen overflow-hidden"
      aria-label={ariaLabel}
    >
      {children}
    </section>
  );
}

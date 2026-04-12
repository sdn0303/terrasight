"use client";

import { Compass, Minus, Plus } from "lucide-react";

interface ZoomOrientControlsProps {
  onZoomIn: () => void;
  onZoomOut: () => void;
  onResetBearing: () => void;
  bottomOffset?: number | undefined;
}

export function ZoomOrientControls({
  onZoomIn,
  onZoomOut,
  onResetBearing,
  bottomOffset = 76,
}: ZoomOrientControlsProps) {
  return (
    // biome-ignore lint/a11y/useAriaPropsSupportedByRole: generic container; aria-label labels the control group for screen readers
    <div
      className="absolute flex flex-col gap-1"
      style={{
        right: 20,
        bottom: bottomOffset,
        zIndex: 15,
      }}
      aria-label="Map view controls"
    >
      <IconButton label="Zoom in" onClick={onZoomIn}>
        <Plus size={14} />
      </IconButton>
      <IconButton label="Zoom out" onClick={onZoomOut}>
        <Minus size={14} />
      </IconButton>
      <IconButton label="Reset bearing" onClick={onResetBearing}>
        <Compass size={14} />
      </IconButton>
    </div>
  );
}

function IconButton({
  label,
  onClick,
  children,
}: {
  label: string;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label={label}
      className="flex h-8 w-8 items-center justify-center rounded-[10px]"
      style={{
        background: "rgba(255,255,255,0.95)",
        color: "var(--neutral-600)",
        boxShadow: "var(--shadow-card-subtle)",
        backdropFilter: "blur(16px)",
      }}
    >
      {children}
    </button>
  );
}

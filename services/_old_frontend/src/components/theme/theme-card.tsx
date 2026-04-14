"use client";

import type { LucideIcon } from "lucide-react";
import { GRADIENT } from "@/lib/theme-tokens";
import type { ThemeId } from "@/lib/themes";

interface ThemeCardProps {
  id: ThemeId;
  label: string;
  description: string;
  layerCount: number;
  icon: LucideIcon;
  active: boolean;
  onToggle: (id: ThemeId) => void;
}

export function ThemeCard({
  id,
  label,
  description,
  layerCount,
  icon: Icon,
  active,
  onToggle,
}: ThemeCardProps) {
  return (
    <button
      type="button"
      aria-pressed={active}
      onClick={() => onToggle(id)}
      className="flex flex-col items-start gap-2 rounded-[14px] p-3.5 text-left transition-colors"
      style={{
        background: active ? GRADIENT.brandTint : "var(--neutral-50)",
        border: active
          ? "1px solid rgba(99, 102, 241, 0.3)"
          : "1px solid var(--neutral-100)",
        boxShadow: active
          ? "inset 0 0 0 1px rgba(99, 102, 241, 0.2)"
          : undefined,
      }}
    >
      <span
        aria-hidden="true"
        className="flex h-8 w-8 items-center justify-center rounded-[10px]"
        style={{
          background: active
            ? "rgba(99, 102, 241, 0.12)"
            : "var(--neutral-100)",
          color: active ? "var(--brand-indigo)" : "var(--neutral-500)",
        }}
      >
        <Icon size={16} aria-hidden="true" />
      </span>
      <span className="flex w-full items-center justify-between">
        <span
          className="text-[11px] font-extrabold"
          style={{
            color: active ? "var(--brand-indigo)" : "var(--neutral-900)",
          }}
        >
          {label}
        </span>
        <span
          className="text-[9px] font-mono"
          style={{ color: "var(--neutral-400)" }}
        >
          {layerCount} layers
        </span>
      </span>
      <span
        className="text-[9px] leading-snug"
        style={{ color: "var(--neutral-500)" }}
      >
        {description}
      </span>
    </button>
  );
}

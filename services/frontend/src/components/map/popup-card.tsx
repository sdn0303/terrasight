"use client";

import type { PopupField } from "@/lib/layers";

interface PopupCardProps {
  layerNameJa: string;
  fields: PopupField[];
  properties: Record<string, unknown>;
}

/**
 * Unified click-inspect popup card for all map layers.
 * Driven by the `popupFields` config in layers.ts — no per-layer templates.
 */
export function PopupCard({ layerNameJa, fields, properties }: PopupCardProps) {
  return (
    <div
      className="rounded px-3 py-2 max-w-[240px] shadow-lg"
      style={{
        background: "var(--bg-secondary)",
        border: "1px solid var(--border-primary)",
        fontFamily: "var(--font-mono)",
        fontSize: "11px",
        color: "var(--text-primary)",
      }}
    >
      <div
        className="text-[10px] tracking-[0.1em] mb-1.5 pb-1"
        style={{
          color: "var(--accent-cyan)",
          borderBottom: "1px solid var(--border-primary)",
        }}
      >
        {layerNameJa}
      </div>
      {fields.map((field) => {
        const value = properties[field.key];
        if (value === undefined || value === null) return null;
        return (
          <div key={field.key} className="flex justify-between gap-2 py-0.5">
            <span style={{ color: "var(--text-secondary)" }}>
              {field.label}
            </span>
            <span style={{ color: "var(--text-primary)" }}>
              {String(value)}
              {field.suffix ? field.suffix : ""}
            </span>
          </div>
        );
      })}
    </div>
  );
}

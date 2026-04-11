"use client";

import { GLOW_SHADOW, GRADIENT } from "@/lib/theme-tokens";
import { useFilterStore } from "@/stores/filter-store";

const ZONES = ["商業", "近商", "住居", "工業"] as const;

export function ZoneChips() {
  const zones = useFilterStore((s) => s.zoning.zones);
  const setZoning = useFilterStore((s) => s.setZoning);

  const toggle = (zone: string) => {
    const next = zones.includes(zone)
      ? zones.filter((z) => z !== zone)
      : [...zones, zone];
    setZoning({ zones: next });
  };

  return (
    <div className="flex flex-wrap gap-1.5">
      {ZONES.map((zone) => {
        const active = zones.includes(zone);
        return (
          <button
            key={zone}
            type="button"
            role="switch"
            aria-checked={active}
            onClick={() => toggle(zone)}
            className="rounded-full px-3 py-1 text-[10px] font-bold transition-all"
            style={
              active
                ? {
                    background: GRADIENT.primary,
                    color: "#fff",
                    boxShadow: GLOW_SHADOW.primarySubtle,
                  }
                : {
                    background: "var(--neutral-100)",
                    color: "var(--neutral-500)",
                  }
            }
          >
            {zone}
          </button>
        );
      })}
    </div>
  );
}

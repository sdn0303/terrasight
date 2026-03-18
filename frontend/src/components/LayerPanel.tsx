"use client";

import { motion } from "framer-motion";
import { TrendingUp, Building2, ShieldAlert, School } from "lucide-react";
import { LAYERS, type ActiveLayers, type LayerDef } from "@/lib/layers";
import type { Dispatch, SetStateAction } from "react";

interface LayerPanelProps {
  activeLayers: ActiveLayers;
  setActiveLayers: Dispatch<SetStateAction<ActiveLayers>>;
}

const CATEGORY_META: Record<
  LayerDef["category"],
  { label: string; icon: React.ElementType; dotColor: string }
> = {
  price: { label: "価格 PRICING", icon: TrendingUp, dotColor: "bg-cyan-400" },
  zoning: { label: "都市計画 ZONING", icon: Building2, dotColor: "bg-amber-400" },
  disaster: { label: "防災 DISASTER", icon: ShieldAlert, dotColor: "bg-red-400" },
  facility: { label: "施設 FACILITIES", icon: School, dotColor: "bg-emerald-400" },
};

const CATEGORY_ORDER: LayerDef["category"][] = ["price", "zoning", "disaster", "facility"];

export default function LayerPanel({ activeLayers, setActiveLayers }: LayerPanelProps) {
  const toggle = (id: string) => {
    setActiveLayers((prev) => ({ ...prev, [id]: !prev[id] }));
  };

  const grouped = CATEGORY_ORDER.map((cat) => ({
    category: cat,
    meta: CATEGORY_META[cat],
    layers: LAYERS.filter((l) => l.category === cat),
  }));

  return (
    <motion.div
      initial={{ x: -320, opacity: 0 }}
      animate={{ x: 0, opacity: 1 }}
      transition={{ duration: 0.4, ease: "easeOut" }}
      className="fixed left-4 top-24 bottom-6 z-[200] w-80 overflow-y-auto rounded-lg border backdrop-blur-md styled-scrollbar"
      style={{
        background: "rgba(10, 10, 15, 0.85)",
        borderColor: "var(--border-primary)",
      }}
    >
      {/* Panel header */}
      <div
        className="sticky top-0 z-10 px-5 py-4 border-b backdrop-blur-sm"
        style={{
          background: "rgba(10, 10, 15, 0.9)",
          borderColor: "var(--border-primary)",
        }}
      >
        <span
          className="text-[10px] font-semibold tracking-widest"
          style={{ color: "var(--text-muted)" }}
        >
          DATA LAYERS
        </span>
      </div>

      {/* Layer groups */}
      <div className="p-3 flex flex-col gap-4">
        {grouped.map(({ category, meta, layers }) => (
          <div key={category}>
            {/* Category header */}
            <div className="flex items-center gap-2 px-2 mb-2">
              <meta.icon
                size={12}
                className="shrink-0"
                style={{ color: "var(--text-muted)" }}
              />
              <span
                className="text-[9px] font-semibold tracking-widest uppercase"
                style={{ color: "var(--text-muted)" }}
              >
                {meta.label}
              </span>
            </div>

            {/* Layer rows */}
            <div className="flex flex-col gap-1">
              {layers.map((layer) => {
                const isOn = activeLayers[layer.id] ?? false;
                return (
                  <button
                    key={layer.id}
                    type="button"
                    onClick={() => toggle(layer.id)}
                    className="flex items-center gap-3 w-full px-3 py-2 rounded-md text-left transition-colors duration-150 cursor-pointer"
                    style={{
                      background: isOn ? "var(--hover-accent)" : "transparent",
                    }}
                  >
                    {/* Colored dot */}
                    <span
                      className={`shrink-0 w-2 h-2 rounded-full ${meta.dotColor}`}
                      style={{ opacity: isOn ? 1 : 0.3 }}
                    />

                    {/* Label */}
                    <div className="flex-1 min-w-0">
                      <div
                        className="text-xs font-medium truncate"
                        style={{ color: isOn ? "var(--text-primary)" : "var(--text-secondary)" }}
                      >
                        {layer.nameJa}
                      </div>
                      <div
                        className="text-[10px] truncate"
                        style={{ color: "var(--text-muted)" }}
                      >
                        {layer.name}
                      </div>
                    </div>

                    {/* ON/OFF badge */}
                    <span
                      className="shrink-0 text-[9px] font-bold tracking-wider px-2 py-0.5 rounded border"
                      style={
                        isOn
                          ? {
                              color: "var(--accent-cyan)",
                              borderColor: "var(--accent-cyan)",
                              boxShadow: "0 0 6px rgba(34, 211, 238, 0.3)",
                            }
                          : {
                              color: "var(--text-muted)",
                              borderColor: "var(--border-primary)",
                            }
                      }
                    >
                      {isOn ? "ON" : "OFF"}
                    </span>
                  </button>
                );
              })}
            </div>
          </div>
        ))}
      </div>
    </motion.div>
  );
}

"use client";

import { AnimatePresence, motion } from "framer-motion";
import { CATEGORIES, LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

export function LayerPanel() {
  const { visibleLayers, toggleLayer } = useMapStore();
  const { layerPanelOpen } = useUIStore();

  return (
    <AnimatePresence>
      {layerPanelOpen && (
        <motion.aside
          initial={{ x: -280 }}
          animate={{ x: 0 }}
          exit={{ x: -280 }}
          transition={{ duration: 0.3 }}
          className="fixed left-0 top-0 bottom-[28px] overflow-y-auto"
          style={{
            width: 280,
            background: "var(--bg-secondary)",
            borderRight: "1px solid var(--border-primary)",
            zIndex: 40,
          }}
          aria-label="Layer controls"
        >
          <div className="px-4 pt-4 pb-2">
            <div
              className="text-xs tracking-[0.15em]"
              style={{ color: "var(--accent-cyan)" }}
            >
              ▸ REALESTATE
            </div>
            <div
              className="text-xs tracking-[0.15em]"
              style={{ color: "var(--accent-cyan)" }}
            >
              &nbsp; INTELLIGENCE
            </div>
          </div>

          {CATEGORIES.map((category) => {
            const categoryLayers = LAYERS.filter(
              (l) => l.category === category.id,
            );
            return (
              <div key={category.id} className="px-4 py-2">
                <div
                  className="text-[9px] tracking-[0.15em] mb-2"
                  style={{ color: "var(--text-muted)" }}
                >
                  ── {category.label} ──
                </div>
                {categoryLayers.map((layer) => {
                  const isActive = visibleLayers.has(layer.id);
                  return (
                    <button
                      key={layer.id}
                      type="button"
                      onClick={() => toggleLayer(layer.id)}
                      className="flex items-center gap-2 w-full px-2 py-1.5 rounded text-left text-xs transition-colors"
                      style={{
                        background: isActive
                          ? "var(--hover-accent)"
                          : "transparent",
                        color: isActive
                          ? "var(--text-primary)"
                          : "var(--text-muted)",
                      }}
                      aria-pressed={isActive}
                      aria-label={`${layer.nameJa} layer toggle`}
                    >
                      <span
                        className="inline-block w-2 h-2 rounded-full"
                        style={{
                          background: isActive
                            ? "var(--accent-cyan)"
                            : "var(--text-muted)",
                        }}
                      />
                      {layer.nameJa}
                    </button>
                  );
                })}
              </div>
            );
          })}
        </motion.aside>
      )}
    </AnimatePresence>
  );
}

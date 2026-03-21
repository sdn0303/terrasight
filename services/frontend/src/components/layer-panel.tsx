"use client";

import { AnimatePresence, motion } from "framer-motion";
import { MenuIcon } from "lucide-react";
import { CATEGORIES, LAYERS } from "@/lib/layers";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetTrigger,
} from "@/components/ui/sheet";
import { useMediaQuery } from "@/hooks/use-media-query";
import { useMapStore } from "@/stores/map-store";
import { useUIStore } from "@/stores/ui-store";

/** Shared layer list content rendered inside both the fixed panel and Sheet variants. */
function LayerPanelContent() {
  const { visibleLayers, toggleLayer } = useMapStore();

  return (
    <>
      <div className="px-4 pt-4 pb-2">
        <div
          className="text-base tracking-[0.05em] font-bold"
          style={{ color: "var(--text-heading)", fontFamily: "var(--font-sans)" }}
        >
          地層
        </div>
        <div
          className="text-[10px] tracking-[0.12em] mt-0.5"
          style={{ color: "var(--text-muted)", fontFamily: "var(--font-mono)" }}
        >
          URBAN STRATIGRAPHY
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
              style={{
                color: "var(--text-muted)",
                fontFamily: "var(--font-mono)",
              }}
            >
              ── {category.labelJa} ──
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
                    fontFamily: "var(--font-sans)",
                  }}
                  aria-pressed={isActive}
                  aria-label={`${layer.nameJa} レイヤー切替`}
                >
                  <span
                    className="inline-block w-2 h-2 rounded-full flex-shrink-0"
                    style={{
                      background: isActive
                        ? layer.color
                        : "var(--text-muted)",
                    }}
                    aria-hidden="true"
                  />
                  {layer.nameJa}
                </button>
              );
            })}
          </div>
        );
      })}
    </>
  );
}

export function LayerPanel() {
  const { layerPanelOpen } = useUIStore();
  const isDesktop = useMediaQuery("(min-width: 1280px)");
  const isMobileOrTablet = !isDesktop;

  // Mobile / tablet: Sheet (bottom on mobile, left side on tablet)
  if (isMobileOrTablet) {
    return (
      <Sheet>
        <SheetTrigger asChild>
          <button
            type="button"
            className="fixed top-4 left-4 z-50 flex items-center justify-center w-9 h-9 rounded"
            style={{
              background: "var(--bg-secondary)",
              border: "1px solid var(--border-primary)",
              color: "var(--accent-cyan)",
            }}
            aria-label="レイヤーコントロールを開く"
          >
            <MenuIcon size={16} />
          </button>
        </SheetTrigger>
        <SheetContent
          side="left"
          className="overflow-y-auto p-0"
          style={{
            background: "var(--bg-secondary)",
            borderRight: "1px solid var(--border-primary)",
            color: "var(--text-primary)",
          }}
        >
          <SheetHeader className="sr-only">
            <SheetTitle>レイヤーコントロール</SheetTitle>
          </SheetHeader>
          <LayerPanelContent />
        </SheetContent>
      </Sheet>
    );
  }

  // Desktop: fixed 280px left panel with animation
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
          aria-label="レイヤーコントロール"
        >
          <LayerPanelContent />
        </motion.aside>
      )}
    </AnimatePresence>
  );
}

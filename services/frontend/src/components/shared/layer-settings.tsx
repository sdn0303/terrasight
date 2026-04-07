"use client";

import { ChevronRightIcon } from "lucide-react";
import { useState } from "react";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { CATEGORIES, LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";

/** Categories expanded by default */
const DEFAULT_EXPANDED = new Set(["value", "risk"]);

/**
 * Standalone layer toggle list grouped by category.
 * Reads visibleLayers and toggles layers via useMapStore.
 * No panel frame — just the toggle content.
 */
export function LayerSettings() {
  const { visibleLayers, toggleLayer } = useMapStore();
  const [expanded, setExpanded] = useState<Set<string>>(
    () => new Set(DEFAULT_EXPANDED),
  );

  const toggleExpanded = (categoryId: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(categoryId)) {
        next.delete(categoryId);
      } else {
        next.add(categoryId);
      }
      return next;
    });
  };

  return (
    <div className="py-1">
      {CATEGORIES.map((category) => {
        const categoryLayers = LAYERS.filter((l) => l.category === category.id);
        const activeCount = categoryLayers.filter((l) =>
          visibleLayers.has(l.id),
        ).length;
        const isOpen = expanded.has(category.id);

        return (
          <Collapsible
            key={category.id}
            open={isOpen}
            onOpenChange={() => toggleExpanded(category.id)}
            className="px-4 py-1"
          >
            <CollapsibleTrigger
              className="flex items-center gap-1.5 w-full py-1.5 text-left"
              aria-expanded={isOpen}
            >
              <ChevronRightIcon
                size={12}
                className="transition-transform duration-200 flex-shrink-0"
                style={{
                  color: "var(--text-muted)",
                  transform: isOpen ? "rotate(90deg)" : "rotate(0deg)",
                }}
              />
              <span
                className="text-[10px] font-medium uppercase tracking-wider flex-1"
                style={{
                  color: "var(--text-muted)",
                }}
              >
                {category.labelJa}
              </span>
              {activeCount > 0 && (
                <span
                  className="text-[9px] px-1.5 py-0.5 rounded-full font-mono"
                  style={{
                    background: "var(--hover-accent)",
                    color: "var(--accent-primary)",
                  }}
                >
                  {activeCount}
                </span>
              )}
            </CollapsibleTrigger>

            <CollapsibleContent>
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
            </CollapsibleContent>
          </Collapsible>
        );
      })}
    </div>
  );
}

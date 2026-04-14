"use client";

import { LeftPanel } from "@/components/layout/left-panel";
import { CATEGORIES, LAYERS } from "@/lib/layers";
import { useMapStore } from "@/stores/map-store";
import { LayerToggleRow } from "./layer-toggle-row";

interface LayerControlPanelProps {
  open: boolean;
  onClose: () => void;
}

export function LayerControlPanel({ open, onClose }: LayerControlPanelProps) {
  const visibleLayers = useMapStore((s) => s.visibleLayers);
  const toggleLayer = useMapStore((s) => s.toggleLayer);

  const activeCount = visibleLayers.size;

  return (
    <LeftPanel
      open={open}
      onClose={onClose}
      title="Data Layers"
      subtitle={`${activeCount} active`}
      badge="LAYERS"
    >
      <div className="space-y-4">
        {CATEGORIES.map((category) => {
          const categoryLayers = LAYERS.filter(
            (l) => l.category === category.id,
          );
          if (categoryLayers.length === 0) return null;
          const categoryActive = categoryLayers.filter((l) =>
            visibleLayers.has(l.id),
          ).length;
          return (
            <section key={category.id} aria-label={category.labelJa}>
              <header
                className="mb-2 flex items-center justify-between text-[9px] font-extrabold uppercase"
                style={{
                  color: "var(--brand-indigo)",
                  letterSpacing: "0.7px",
                }}
              >
                <span>● {category.label}</span>
                <span style={{ color: "var(--neutral-400)" }}>
                  {categoryActive}/{categoryLayers.length}
                </span>
              </header>
              <div className="flex flex-col gap-0.5">
                {categoryLayers.map((layer) => (
                  <LayerToggleRow
                    key={layer.id}
                    id={layer.id}
                    label={layer.nameJa}
                    swatch={layer.color}
                    checked={visibleLayers.has(layer.id)}
                    onToggle={toggleLayer}
                  />
                ))}
              </div>
            </section>
          );
        })}
      </div>
    </LeftPanel>
  );
}

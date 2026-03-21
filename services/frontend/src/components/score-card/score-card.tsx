"use client";

import { AnimatePresence, motion } from "framer-motion";
import { ScoreGauge } from "./score-gauge";
import { ComponentBar } from "./component-bar";
import { Sparkline } from "./sparkline";
import { useScore } from "@/features/score/api/use-score";
import { useTrend } from "@/features/trend/api/use-trend";
import { useMediaQuery } from "@/hooks/use-media-query";
import { useMapStore } from "@/stores/map-store";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";

/** Inner content shared between fixed panel and Sheet. */
function ScoreCardContent({
  onClose,
}: {
  onClose: () => void;
}) {
  const { selectedFeature } = useMapStore();
  const lat = selectedFeature?.coordinates[1] ?? null;
  const lng = selectedFeature?.coordinates[0] ?? null;
  const { data: score, isLoading: scoreLoading } = useScore(lat, lng);
  const { data: trendData } = useTrend(lat, lng);

  if (!selectedFeature) return null;

  return (
    <div className="overflow-y-auto flex-1">
      {/* Header */}
      <div
        className="flex items-center justify-between px-4 py-3 border-b"
        style={{ borderColor: "var(--border-primary)" }}
      >
        <span
          className="text-[9px] tracking-[0.15em]"
          style={{ color: "var(--accent-cyan)" }}
        >
          PROPERTY INTEL
        </span>
        <button
          type="button"
          onClick={onClose}
          className="text-xs"
          style={{ color: "var(--text-muted)" }}
          aria-label="Close score card"
        >
          &times;
        </button>
      </div>

      <div className="p-4 space-y-4">
        {/* Location */}
        <div>
          <div
            className="text-[9px] tracking-[0.15em] mb-1"
            style={{ color: "var(--text-muted)" }}
          >
            LOCATION
          </div>
          <div
            className="text-xs"
            style={{ color: "var(--text-primary)" }}
          >
            {String(
              selectedFeature.properties["address"] ??
                `${lat?.toFixed(4)}°N, ${lng?.toFixed(4)}°E`,
            )}
          </div>
        </div>

        {/* Investment Score */}
        {scoreLoading ? (
          <div className="space-y-2">
            <Skeleton className="h-4 w-32" />
            <Skeleton className="h-24 w-full" />
          </div>
        ) : score ? (
          <div
            className="rounded-lg p-3"
            style={{ background: "var(--bg-tertiary)" }}
          >
            <div
              className="text-[9px] tracking-[0.15em] mb-2"
              style={{ color: "var(--text-muted)" }}
            >
              INVESTMENT SCORE
            </div>
            <ScoreGauge score={score.score} />
            <div className="space-y-1.5 mt-3">
              <ComponentBar
                label="trend"
                value={score.components.trend.value}
                max={score.components.trend.max}
              />
              <ComponentBar
                label="risk"
                value={score.components.risk.value}
                max={score.components.risk.max}
              />
              <ComponentBar
                label="access"
                value={score.components.access.value}
                max={score.components.access.max}
              />
              <ComponentBar
                label="yield"
                value={score.components.yield_potential.value}
                max={score.components.yield_potential.max}
              />
            </div>
          </div>
        ) : null}

        {/* Pricing */}
        {selectedFeature.properties["price_per_sqm"] !== undefined && (
          <div
            className="rounded-lg p-3"
            style={{ background: "var(--bg-tertiary)" }}
          >
            <div
              className="text-[9px] tracking-[0.15em] mb-2"
              style={{ color: "var(--text-muted)" }}
            >
              PRICING
            </div>
            <div className="flex justify-between text-xs">
              <span style={{ color: "var(--text-secondary)" }}>
                per sqm
              </span>
              <span style={{ color: "var(--accent-cyan)" }}>
                ¥
                {Number(
                  selectedFeature.properties["price_per_sqm"],
                ).toLocaleString()}
              </span>
            </div>
          </div>
        )}

        {/* Price Trend Sparkline */}
        {trendData && <Sparkline trend={trendData} />}

        {/* Disclaimer */}
        {score && (
          <div
            className="text-[9px]"
            style={{ color: "var(--text-muted)" }}
          >
            {score.metadata.disclaimer}
          </div>
        )}
      </div>
    </div>
  );
}

export function ScoreCard() {
  const { selectedFeature, selectFeature } = useMapStore();
  const isDesktop = useMediaQuery("(min-width: 1280px)");
  const isTablet = useMediaQuery("(min-width: 768px)");
  const isMobile = !isTablet;

  const handleClose = () => selectFeature(null);

  // Mobile: bottom Sheet
  if (isMobile) {
    return (
      <Sheet open={selectedFeature !== null} onOpenChange={(open) => { if (!open) handleClose(); }}>
        <SheetContent
          side="bottom"
          showCloseButton={false}
          className="h-[70vh] p-0 flex flex-col"
          style={{
            background: "rgba(10, 10, 15, 0.97)",
            borderTop: "1px solid var(--border-primary)",
            color: "var(--text-primary)",
          }}
        >
          <SheetHeader className="sr-only">
            <SheetTitle>Property Score Card</SheetTitle>
          </SheetHeader>
          <ScoreCardContent onClose={handleClose} />
        </SheetContent>
      </Sheet>
    );
  }

  // Tablet and desktop: fixed right panel (280px tablet, 320px desktop)
  const panelWidth = isDesktop ? 320 : 280;

  return (
    <AnimatePresence>
      {selectedFeature && (
        <motion.aside
          initial={{ x: panelWidth }}
          animate={{ x: 0 }}
          exit={{ x: panelWidth }}
          transition={{ duration: 0.3 }}
          className="fixed right-4 top-4 bottom-[148px] overflow-y-auto rounded-lg"
          style={{
            width: panelWidth,
            background: "rgba(10, 10, 15, 0.9)",
            backdropFilter: "blur(12px)",
            border: "1px solid var(--border-primary)",
            zIndex: 50,
          }}
          aria-label="Property score card"
        >
          <ScoreCardContent onClose={handleClose} />
        </motion.aside>
      )}
    </AnimatePresence>
  );
}

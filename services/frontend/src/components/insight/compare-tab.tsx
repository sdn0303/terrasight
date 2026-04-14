"use client";

import { useQueries } from "@tanstack/react-query";
import { DiffTable } from "@/components/compare/diff-table";
import { RadarComparison } from "@/components/compare/radar-chart";
import { typedGet } from "@/lib/api";
import { TlsResponse } from "@/lib/api/schemas/score";
import { useUIStore } from "@/stores/ui-store";

/**
 * Drawer tab for N-point TLS comparison. Fetches scores for each
 * `comparePoints` entry via `useQueries`, then renders the existing
 * {@link RadarComparison} + {@link DiffTable} components once at least
 * two points have loaded. The list of compare points is owned by
 * `ui-store`; the tab is purely a reader + dispatcher.
 */
export function CompareTab() {
  const comparePoints = useUIStore((s) => s.comparePoints);
  const removeComparePoint = useUIStore((s) => s.removeComparePoint);
  const resetCompare = useUIStore((s) => s.resetCompare);

  const results = useQueries({
    queries: comparePoints.map((pt) => ({
      queryKey: ["score", pt.lat, pt.lng, "balance"] as const,
      queryFn: () =>
        typedGet(TlsResponse, "api/v1/score", {
          lat: String(pt.lat),
          lng: String(pt.lng),
          preset: "balance",
        }),
      staleTime: 60_000,
    })),
  });

  const readyScores = results
    .map((r) => r.data)
    .filter((s): s is NonNullable<typeof s> => s != null);

  if (comparePoints.length === 0) {
    return <EmptyCompareState />;
  }

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-2">
        {comparePoints.map((pt, i) => (
          <div
            key={`${pt.lat}-${pt.lng}`}
            className="rounded-lg px-2.5 py-1.5 text-[10px]"
            style={{
              background: "var(--neutral-50)",
              border: "1px solid var(--neutral-100)",
            }}
          >
            <div className="flex items-center gap-2">
              <span style={{ color: "var(--neutral-900)", fontWeight: 700 }}>
                Point {String.fromCharCode(65 + i)}
              </span>
              <button
                type="button"
                onClick={() => removeComparePoint(i)}
                aria-label={`Remove point ${i + 1}`}
                style={{ color: "var(--neutral-400)" }}
              >
                ×
              </button>
            </div>
            <div style={{ color: "var(--neutral-500)", fontSize: 9 }}>
              {pt.address}
            </div>
          </div>
        ))}
      </div>

      {readyScores.length >= 2 ? (
        <>
          <RadarComparison
            axesList={readyScores.map((s) => s.axes)}
            labels={comparePoints.map(
              (_, i) => `Point ${String.fromCharCode(65 + i)}`,
            )}
          />
          <DiffTable
            axesList={readyScores.map((s) => s.axes)}
            tlsScores={readyScores.map((s) => s.tls.score)}
          />
        </>
      ) : (
        <div
          className="p-4 text-center text-[10px]"
          style={{ color: "var(--neutral-500)" }}
        >
          {comparePoints.length < 2
            ? "Select at least 2 properties from the Opportunities table to compare."
            : "Loading scores..."}
        </div>
      )}

      <button
        type="button"
        onClick={resetCompare}
        className="w-full rounded-[12px] py-2 text-[10px] font-bold"
        style={{
          background: "var(--neutral-100)",
          color: "var(--neutral-600)",
        }}
      >
        Clear comparison
      </button>
    </div>
  );
}

function EmptyCompareState() {
  return (
    <div
      className="flex h-full items-center justify-center p-6 text-center text-[10px]"
      style={{ color: "var(--neutral-500)" }}
    >
      Select 2-3 properties from the Opportunities table to compare their TLS
      axes side by side.
    </div>
  );
}

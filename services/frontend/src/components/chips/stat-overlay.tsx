

import { PageChip } from "@/components/layout/page-chip";

interface StatOverlayProps {
  matches: number;
  deltaPct: number | null;
  avgTls: number;
  hotCount: number;
}

export function StatOverlay({
  matches,
  deltaPct,
  avgTls,
  hotCount,
}: StatOverlayProps) {
  return (
    <PageChip anchor="top-right" aria-label="Area statistics">
      <div>
        <div
          className="text-[9px] font-extrabold uppercase"
          style={{
            color: "var(--brand-indigo)",
            letterSpacing: "0.6px",
          }}
        >
          ● Matches
        </div>
        <div className="flex items-baseline gap-1.5">
          <span
            className="text-[20px] font-extrabold"
            style={{ color: "var(--neutral-900)" }}
          >
            {matches.toLocaleString("en-US")}
          </span>
          {deltaPct !== null && (
            <span
              className="text-[9px] font-bold"
              style={{
                color:
                  deltaPct > 0
                    ? "var(--score-good-start)"
                    : deltaPct < 0
                      ? "var(--score-bad-start)"
                      : "var(--neutral-400)",
              }}
            >
              {deltaPct > 0 ? "▲ +" : deltaPct < 0 ? "▼ " : "— "}
              {deltaPct.toFixed(1)}%
            </span>
          )}
        </div>
        <div
          className="mt-0.5 text-[9px]"
          style={{ color: "var(--neutral-400)" }}
        >
          Avg TLS <b style={{ color: "var(--brand-indigo)" }}>{avgTls}</b> · Hot{" "}
          <b style={{ color: "var(--signal-hot-start)" }}>{hotCount}</b>
        </div>
      </div>
    </PageChip>
  );
}

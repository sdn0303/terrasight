import type { OpportunityRiskLevel } from "@/lib/api/schemas/opportunities";

interface RiskPillProps {
  level: OpportunityRiskLevel;
}

const STYLES: Record<
  OpportunityRiskLevel,
  { bg: string; fg: string; label: string }
> = {
  low: { bg: "#d1fae5", fg: "#047857", label: "LOW" },
  mid: { bg: "#fef3c7", fg: "#92400e", label: "MID" },
  high: { bg: "#fee2e2", fg: "#991b1b", label: "HIGH" },
};

/**
 * Compact risk badge (LOW/MID/HIGH) with a pastel background per tier.
 */
export function RiskPill({ level }: RiskPillProps) {
  const s = STYLES[level];
  return (
    <span
      className="inline-block rounded-full text-[9px] font-bold uppercase"
      style={{
        background: s.bg,
        color: s.fg,
        padding: "3px 10px",
      }}
    >
      {s.label}
    </span>
  );
}

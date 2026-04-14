import type { Opportunity, OpportunityRiskLevel } from "@/lib/api/schemas/opportunities";

export type OpportunityRow = Opportunity;

export interface ColumnDef {
  key: string;
  label: string;
  width: string;
  render: (row: OpportunityRow) => React.ReactNode;
}

const RISK_BADGE: Record<OpportunityRiskLevel, { label: string; bg: string; color: string }> = {
  low: { label: "低", bg: "rgba(34,197,94,0.15)", color: "#16a34a" },
  mid: { label: "中", bg: "rgba(234,179,8,0.15)", color: "#ca8a04" },
  high: { label: "高", bg: "rgba(239,68,68,0.15)", color: "#dc2626" },
};

function TlsBadge({ score }: { score: number }) {
  const hue = Math.round((score / 100) * 120); // 0=red, 120=green
  return (
    <span
      className="inline-block rounded px-1.5 py-0.5 text-[11px] font-bold tabular-nums"
      style={{
        background: `hsl(${hue} 70% 92%)`,
        color: `hsl(${hue} 60% 35%)`,
      }}
      aria-label={`TLSスコア ${score}`}
    >
      {score}
    </span>
  );
}

function RiskBadge({ level }: { level: OpportunityRiskLevel }) {
  const cfg = RISK_BADGE[level];
  return (
    <span
      className="inline-block rounded px-1.5 py-0.5 text-[11px] font-medium"
      style={{ background: cfg.bg, color: cfg.color }}
      aria-label={`リスク ${cfg.label}`}
    >
      {cfg.label}
    </span>
  );
}

function TrendCell({ pct }: { pct: number }) {
  const isUp = pct >= 0;
  return (
    <span
      className="tabular-nums text-[12px] font-medium"
      style={{ color: isUp ? "#16a34a" : "#dc2626" }}
      aria-label={`トレンド ${isUp ? "上昇" : "下落"} ${Math.abs(pct).toFixed(1)}%`}
    >
      {isUp ? "↑" : "↓"}
      {Math.abs(pct).toFixed(1)}%
    </span>
  );
}

export const opportunityColumns: readonly ColumnDef[] = [
  {
    key: "address",
    label: "所在地",
    width: "flex",
    render: (row) => (
      <span
        className="truncate text-[12px]"
        style={{ color: "var(--panel-text-primary)" }}
        title={row.address}
      >
        {row.address}
      </span>
    ),
  },
  {
    key: "tls",
    label: "TLS",
    width: "60px",
    render: (row) => <TlsBadge score={row.tls} />,
  },
  {
    key: "price_per_sqm",
    label: "地価",
    width: "90px",
    render: (row) => (
      <span
        className="tabular-nums text-[12px]"
        style={{ color: "var(--panel-text-primary)" }}
        aria-label={`地価 ${row.price_per_sqm.toLocaleString("ja-JP")}円`}
      >
        ¥{(row.price_per_sqm / 1000).toLocaleString("ja-JP", { maximumFractionDigits: 1 })}k
      </span>
    ),
  },
  {
    key: "risk_level",
    label: "リスク",
    width: "70px",
    render: (row) => <RiskBadge level={row.risk_level} />,
  },
  {
    key: "trend_pct",
    label: "トレンド",
    width: "70px",
    render: (row) => <TrendCell pct={row.trend_pct} />,
  },
  {
    key: "station",
    label: "最寄駅",
    width: "120px",
    render: (row) => (
      <span
        className="truncate text-[12px]"
        style={{ color: "var(--panel-text-secondary)" }}
        title={row.station?.name ?? "—"}
      >
        {row.station != null
          ? `${row.station.name} ${(row.station.distance_m / 1000).toFixed(1)}km`
          : "—"}
      </span>
    ),
  },
] as const;

"use client";

interface Props {
  lat: number;
  lng: number;
  featureProperties?: Record<string, unknown>;
}

type RiskLevel = "high" | "medium" | "low" | "unknown";

const RISK_LEVEL_LABEL: Record<RiskLevel, string> = {
  high: "高リスク",
  medium: "中リスク",
  low: "低リスク",
  unknown: "不明",
};

const RISK_LEVEL_COLOR: Record<RiskLevel, string> = {
  high: "#ef4444",
  medium: "#f97316",
  low: "#22c55e",
  unknown: "#94a3b8",
};

function toRiskLevel(value: unknown): RiskLevel {
  if (value === "high" || value === "medium" || value === "low") return value;
  return "unknown";
}

function RiskBadge({ level }: { level: RiskLevel }) {
  return (
    <span
      className="inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold text-white"
      style={{ backgroundColor: RISK_LEVEL_COLOR[level] }}
    >
      {RISK_LEVEL_LABEL[level]}
    </span>
  );
}

function PropertyRow({ label, value }: { label: string; value: string }) {
  return (
    <tr>
      <td
        className="py-1.5 pr-3 text-xs w-1/2"
        style={{ color: "var(--panel-text-secondary)" }}
      >
        {label}
      </td>
      <td
        className="py-1.5 text-xs font-medium"
        style={{ color: "var(--panel-text-primary)" }}
      >
        {value}
      </td>
    </tr>
  );
}

function formatPropertyValue(value: unknown): string {
  if (value === null || value === undefined) return "—";
  if (typeof value === "number") return value.toLocaleString();
  if (typeof value === "string") return value;
  if (typeof value === "boolean") return value ? "あり" : "なし";
  return String(value);
}

export function HazardDetail({ featureProperties }: Props) {
  const riskLevel = toRiskLevel(featureProperties?.risk_level);

  // Build display rows from feature properties, excluding internal fields
  const EXCLUDE_KEYS = new Set(["risk_level"]);
  const rows: Array<{ label: string; value: string }> = [];

  if (featureProperties !== undefined) {
    for (const [key, val] of Object.entries(featureProperties)) {
      if (EXCLUDE_KEYS.has(key)) continue;
      rows.push({ label: key, value: formatPropertyValue(val) });
    }
  }

  return (
    <div className="space-y-4">
      {/* Risk badge */}
      <div>
        <p
          className="mb-2 text-[10px] font-semibold uppercase tracking-wider"
          style={{ color: "var(--panel-text-secondary)" }}
        >
          総合リスク
        </p>
        <RiskBadge level={riskLevel} />
      </div>

      {/* Property details table */}
      {rows.length > 0 && (
        <div>
          <p
            className="mb-2 text-[10px] font-semibold uppercase tracking-wider"
            style={{ color: "var(--panel-text-secondary)" }}
          >
            詳細データ
          </p>
          <table className="w-full">
            <tbody>
              {rows.map((row) => (
                <PropertyRow
                  key={row.label}
                  label={row.label}
                  value={row.value}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}

      {rows.length === 0 && featureProperties === undefined && (
        <p className="text-xs" style={{ color: "var(--panel-text-secondary)" }}>
          地図上のハザードレイヤーをクリックして詳細を確認してください
        </p>
      )}

      {/* WASM integration note */}
      <p
        className="text-[10px] border-t pt-3"
        style={{
          color: "var(--panel-text-secondary)",
          borderColor: "var(--panel-border)",
        }}
      >
        ※ 空間クエリはフェーズ2で統合予定
      </p>
    </div>
  );
}

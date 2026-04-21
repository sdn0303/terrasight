/**
 * 公示地価セクション。価格 + 変動率 + 建蔽率/容積率。
 * Ref: DESIGN.md Sec 5.3 item 6
 */
export function LandPriceSection() {
  // TODO: useLandPriceAggregation hook を接続
  const price = 852000;
  const changePct = 3.2;

  return (
    <div className="p-4">
      <p
        className="text-xs font-medium tracking-wider mb-2"
        style={{ color: "var(--ts-text-muted)", letterSpacing: "0.5px" }}
      >
        公示地価
      </p>

      <div className="flex items-baseline gap-2">
        <span
          className="text-2xl font-medium font-mono"
          style={{ color: "var(--ts-text-primary)" }}
        >
          {(price / 10000).toFixed(1)}
        </span>
        <span className="text-xs" style={{ color: "var(--ts-text-muted)" }}>
          万円/m²
        </span>
        <span
          className="text-xs px-1.5 py-0.5 rounded font-mono"
          style={{
            background:
              changePct >= 0
                ? "var(--ts-score-excellent)"
                : "var(--ts-score-danger)",
            color: "#111111",
          }}
        >
          {changePct >= 0 ? "+" : ""}
          {changePct}%
        </span>
      </div>

      {/* Building coverage / Floor area ratio */}
      <div className="mt-3 flex gap-4">
        <MetricItem label="建蔽率" value="60%" />
        <MetricItem label="容積率" value="200%" />
        <MetricItem label="用途地域" value="商業" />
      </div>
    </div>
  );
}

function MetricItem({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-[10px]" style={{ color: "var(--ts-text-dim)" }}>
        {label}
      </p>
      <p
        className="text-xs font-mono font-medium"
        style={{ color: "var(--ts-text-secondary)" }}
      >
        {value}
      </p>
    </div>
  );
}

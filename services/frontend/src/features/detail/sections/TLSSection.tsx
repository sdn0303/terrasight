/**
 * TLS スコアセクション。大きな数値 + 6軸分解。
 * Ref: DESIGN.md Sec 5.3 items 3-4
 */
export function TLSSection() {
  // TODO: useScore hook を接続してリアルデータを表示
  const score = 82.4;
  const axes = [
    { label: "地価トレンド", value: 85 },
    { label: "リスク", value: 72 },
    { label: "利便性", value: 90 },
    { label: "生活環境", value: 78 },
    { label: "人口動態", value: 81 },
    { label: "地盤安全性", value: 88 },
  ];

  return (
    <div className="p-4">
      <p
        className="text-xs font-medium tracking-wider mb-2"
        style={{ color: "var(--ts-text-muted)", letterSpacing: "0.5px" }}
      >
        TLS 総合スコア
      </p>
      <div className="flex items-baseline gap-2">
        <span
          className="text-[40px] font-light tracking-tight"
          style={{
            color: "var(--ts-score-excellent)",
            fontFamily: "var(--font-sans)",
            letterSpacing: "-1px",
          }}
        >
          {score}
        </span>
        <span
          className="text-xs px-1.5 py-0.5 rounded"
          style={{
            background: "var(--ts-score-excellent)",
            color: "#111111",
            fontFamily: "var(--font-mono)",
          }}
        >
          +2.1 前年比
        </span>
      </div>

      {/* 6-axis breakdown */}
      <div className="mt-4 flex flex-col gap-2">
        {axes.map((axis) => (
          <div key={axis.label} className="flex items-center gap-2">
            <span
              className="text-xs w-20 shrink-0"
              style={{ color: "var(--ts-text-muted)" }}
            >
              {axis.label}
            </span>
            <div
              className="flex-1 h-1.5 rounded-full overflow-hidden"
              style={{ background: "var(--ts-border-subtle)" }}
            >
              <div
                className="h-full rounded-full transition-all"
                style={{
                  width: `${axis.value}%`,
                  background: getScoreColor(axis.value),
                }}
              />
            </div>
            <span
              className="text-xs w-8 text-right font-mono"
              style={{ color: "var(--ts-text-secondary)" }}
            >
              {axis.value}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}

function getScoreColor(value: number): string {
  if (value >= 80) return "var(--ts-score-excellent)";
  if (value >= 60) return "var(--ts-score-good)";
  if (value >= 40) return "var(--ts-score-average)";
  if (value >= 20) return "var(--ts-score-caution)";
  return "var(--ts-score-danger)";
}

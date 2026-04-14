interface ScoreGaugeProps {
  score: number; // 0-100
}

function getScoreColor(score: number): string {
  if (score < 34) return "var(--accent-danger)";
  if (score < 67) return "var(--accent-warning)";
  return "var(--accent-success)";
}

export function ScoreGauge({ score }: ScoreGaugeProps) {
  const color = getScoreColor(score);
  const clampedScore = Math.max(0, Math.min(100, score));
  const angle = (clampedScore / 100) * 180;
  const rad = (angle * Math.PI) / 180;
  const r = 60;
  const cx = 80;
  const cy = 75;
  const x = cx + r * Math.cos(Math.PI - rad);
  const y = cy - r * Math.sin(Math.PI - rad);
  const largeArc = angle > 90 ? 1 : 0;

  return (
    <svg
      viewBox="0 0 160 100"
      className="w-full"
      aria-label={`Investment score: ${score}`}
      role="img"
    >
      {/* Track */}
      <path
        d={`M ${cx - r} ${cy} A ${r} ${r} 0 0 1 ${cx + r} ${cy}`}
        fill="none"
        stroke="var(--bg-tertiary)"
        strokeWidth={8}
      />
      {/* Arc fill */}
      {clampedScore > 0 && (
        <path
          d={`M ${cx - r} ${cy} A ${r} ${r} 0 ${largeArc} 1 ${x} ${y}`}
          fill="none"
          stroke={color}
          strokeWidth={8}
          strokeLinecap="round"
        />
      )}
      {/* Score number */}
      <text
        x={cx}
        y={cy - 10}
        textAnchor="middle"
        fill={color}
        fontSize={28}
        fontFamily="var(--font-mono)"
        fontWeight="bold"
      >
        {Math.round(score)}
      </text>
      {/* Scale labels */}
      <text
        x={cx - r}
        y={cy + 15}
        textAnchor="middle"
        fill="var(--text-muted)"
        fontSize={9}
      >
        0
      </text>
      <text
        x={cx}
        y={cy + 15}
        textAnchor="middle"
        fill="var(--text-muted)"
        fontSize={9}
      >
        50
      </text>
      <text
        x={cx + r}
        y={cy + 15}
        textAnchor="middle"
        fill="var(--text-muted)"
        fontSize={9}
      >
        100
      </text>
    </svg>
  );
}

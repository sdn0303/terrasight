import { GLOW_SHADOW, scoreGradient } from "@/lib/theme-tokens";

interface ScoreChipProps {
  value: number;
  size?: "xs" | "sm" | "md";
}

/**
 * Pill rendering a 0-100 TLS score with color band chosen by
 * {@link scoreGradient}. Emits a glow shadow whose intensity matches the
 * band (success / brand / warn / danger).
 */
export function ScoreChip({ value, size = "sm" }: ScoreChipProps) {
  const glow =
    value >= 80
      ? GLOW_SHADOW.success
      : value >= 60
        ? GLOW_SHADOW.primarySubtle
        : value >= 40
          ? GLOW_SHADOW.warn
          : GLOW_SHADOW.danger;
  const fontSize = size === "xs" ? 9 : size === "md" ? 12 : 10;
  const padding =
    size === "xs" ? "3px 10px" : size === "md" ? "5px 14px" : "4px 12px";

  return (
    <span
      className="inline-block rounded-full font-extrabold text-white"
      style={{
        background: scoreGradient(value),
        boxShadow: glow,
        fontSize,
        padding,
      }}
    >
      {value}
    </span>
  );
}

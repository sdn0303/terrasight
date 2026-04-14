"use client";

// TODO: replace with lib/i18n.ts (Task 1.6)
function useTranslations(_ns?: string) {
  return (key: string) => key;
}

type TlsGrade = "S" | "A" | "B" | "C" | "D" | "E";

const GRADE_COLORS: Record<TlsGrade, string> = {
  S: "#10b981",
  A: "#22c55e",
  B: "#eab308",
  C: "#f97316",
  D: "#ef4444",
  E: "#991b1b",
};

interface TlsScoreHeaderProps {
  score: number;
  grade: TlsGrade;
  label: string;
}

export function TlsScoreHeader({ score, grade, label }: TlsScoreHeaderProps) {
  const t = useTranslations("tls");

  return (
    <div className="px-4 py-4 text-center">
      <div className="text-[9px] font-semibold tracking-widest uppercase text-ds-text-muted mb-2">
        {t("score").toUpperCase()}
      </div>
      <div
        className="text-4xl font-bold"
        style={{ color: GRADE_COLORS[grade] }}
      >
        {Math.round(score)}
      </div>
      <div className="flex items-center justify-center gap-2 mt-1">
        <span
          className="text-lg font-bold"
          style={{ color: GRADE_COLORS[grade] }}
        >
          {grade}
        </span>
        <span className="text-xs text-ds-text-muted">{label}</span>
      </div>
    </div>
  );
}

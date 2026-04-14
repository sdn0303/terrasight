"use client";

// TODO: replace with lib/i18n.ts (Task 1.6)
function useTranslations(_ns?: string) {
  return (key: string) => key;
}

import type { TlsResponse } from "@/lib/schemas";

const AXIS_KEYS = [
  "disaster",
  "terrain",
  "livability",
  "future",
  "price",
] as const;
const COLORS = ["text-ds-accent-primary", "text-amber-400", "text-emerald-400"];

interface DiffTableProps {
  axesList: TlsResponse["axes"][];
  tlsScores: number[];
}

export function DiffTable({ axesList, tlsScores }: DiffTableProps) {
  const t = useTranslations("axis");
  const pointCount = axesList.length;

  const rows = AXIS_KEYS.map((key) => ({
    key,
    label: t(key),
    scores: axesList.map((axes) => Math.round(axes?.[key]?.score ?? 0)),
  }));

  const bestIndex = (scores: number[]) => {
    let maxIdx = 0;
    for (let i = 1; i < scores.length; i++) {
      if ((scores[i] ?? 0) > (scores[maxIdx] ?? 0)) maxIdx = i;
    }
    return maxIdx;
  };

  return (
    <div className="px-4">
      <table className="w-full text-[10px]">
        <thead>
          <tr className="text-ds-text-muted">
            <th className="text-left py-1 font-normal" />
            {Array.from({ length: pointCount }, (_, i) => (
              <th
                key={`h${i}`}
                className={`text-right py-1 font-normal ${COLORS[i % COLORS.length]}`}
              >
                {String.fromCharCode(65 + i)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map(({ key, label, scores }) => {
            const best = bestIndex(scores);
            return (
              <tr key={key} className="text-ds-text-secondary">
                <td className="py-0.5">{label}</td>
                {scores.map((s, i) => (
                  <td
                    key={`${key}-${i}`}
                    className={`text-right font-mono ${i === best ? COLORS[i % COLORS.length] : ""}`}
                  >
                    {s}
                  </td>
                ))}
              </tr>
            );
          })}
          <tr className="border-t border-ds-border-primary text-ds-text-heading">
            <td className="py-1 font-medium">TLS</td>
            {tlsScores.map((s, i) => {
              const best = bestIndex(tlsScores);
              return (
                <td
                  key={`tls-${i}`}
                  className={`text-right font-mono font-medium ${i === best ? COLORS[i % COLORS.length] : ""}`}
                >
                  {Math.round(s)}
                </td>
              );
            })}
          </tr>
        </tbody>
      </table>
    </div>
  );
}

"use client";

import { useTranslations } from "next-intl";
import type { TlsResponse } from "@/lib/schemas";

const AXIS_KEYS = [
  "disaster",
  "terrain",
  "livability",
  "future",
  "price",
] as const;

interface DiffTableProps {
  axesA: TlsResponse["axes"];
  axesB: TlsResponse["axes"];
  tlsA: number;
  tlsB: number;
}

export function DiffTable({ axesA, axesB, tlsA, tlsB }: DiffTableProps) {
  const t = useTranslations("axis");

  const rows = AXIS_KEYS.map((key) => ({
    key,
    label: t(key),
    a: Math.round(axesA[key].score),
    b: Math.round(axesB[key].score),
  }));

  return (
    <div className="px-4">
      <table className="w-full text-[10px]">
        <thead>
          <tr className="text-ds-text-muted">
            <th className="text-left py-1 font-normal" />
            <th className="text-right py-1 font-normal text-ds-accent-primary">
              A
            </th>
            <th className="text-right py-1 font-normal text-amber-400">B</th>
            <th className="text-right py-1 font-normal">Delta</th>
          </tr>
        </thead>
        <tbody>
          {rows.map(({ key, label, a, b }) => {
            const delta = a - b;
            return (
              <tr key={key} className="text-ds-text-secondary">
                <td className="py-0.5">{label}</td>
                <td className="text-right font-mono">{a}</td>
                <td className="text-right font-mono">{b}</td>
                <td
                  className={`text-right font-mono ${delta > 0 ? "text-ds-accent-primary" : delta < 0 ? "text-amber-400" : "text-ds-text-muted"}`}
                >
                  {delta > 0
                    ? `A+${delta}`
                    : delta < 0
                      ? `B+${Math.abs(delta)}`
                      : "="}
                </td>
              </tr>
            );
          })}
          <tr className="border-t border-ds-border-primary text-ds-text-heading">
            <td className="py-1 font-medium">TLS</td>
            <td className="text-right font-mono font-medium">
              {Math.round(tlsA)}
            </td>
            <td className="text-right font-mono font-medium">
              {Math.round(tlsB)}
            </td>
            <td
              className={`text-right font-mono font-medium ${tlsA > tlsB ? "text-ds-accent-primary" : tlsB > tlsA ? "text-amber-400" : "text-ds-text-muted"}`}
            >
              {tlsA > tlsB
                ? `A+${Math.round(tlsA - tlsB)}`
                : tlsB > tlsA
                  ? `B+${Math.round(tlsB - tlsA)}`
                  : "="}
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  );
}

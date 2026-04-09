"use client";

import type { TlsResponse } from "@/lib/schemas";

interface InfraProximityProps {
  livabilityAxis: TlsResponse["axes"]["livability"];
}

function getNumber(detail: Record<string, unknown>, key: string): number {
  const val = detail[key];
  return typeof val === "number" ? val : 0;
}

export function InfraProximity({ livabilityAxis }: InfraProximityProps) {
  const subs = livabilityAxis.sub;
  const medical = subs.find((s) => s.id === "medical");
  const education = subs.find((s) => s.id === "education");

  return (
    <div className="px-4 py-2">
      <div className="text-[9px] font-semibold tracking-wider uppercase text-ds-text-muted mb-2">
        NEARBY INFRASTRUCTURE
      </div>
      <div className="grid grid-cols-2 gap-2 text-[10px]">
        {medical && (
          <div className="rounded-lg bg-ds-bg-tertiary/50 p-2">
            <div className="text-ds-text-muted">Medical</div>
            <div className="text-ds-text-primary font-medium">
              {getNumber(medical.detail, "hospital_count")} hospitals
            </div>
            <div className="text-ds-text-muted">
              {getNumber(medical.detail, "clinic_count")} clinics
            </div>
          </div>
        )}
        {education && (
          <div className="rounded-lg bg-ds-bg-tertiary/50 p-2">
            <div className="text-ds-text-muted">Education</div>
            <div className="text-ds-text-primary font-medium">
              {getNumber(education.detail, "count_800m")} schools (800m)
            </div>
          </div>
        )}
      </div>
    </div>
  );
}

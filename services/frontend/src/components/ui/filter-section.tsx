"use client";

import type { ReactNode } from "react";

interface FilterSectionProps {
  title: string;
  required?: boolean;
  children: ReactNode;
}

/**
 * Generic section header + children used in Finder and Layers panels.
 * Header is a uppercase label with brand color and optional "Required" tag.
 */
export function FilterSection({
  title,
  required,
  children,
}: FilterSectionProps) {
  return (
    <div className="mb-5">
      <div className="mb-2 flex items-center justify-between">
        <div
          className="text-[9px] font-extrabold uppercase"
          style={{
            color: "var(--brand-indigo)",
            letterSpacing: "0.7px",
          }}
        >
          {title}
        </div>
        {required && (
          <div className="text-[9px]" style={{ color: "var(--neutral-400)" }}>
            Required
          </div>
        )}
      </div>
      {children}
    </div>
  );
}

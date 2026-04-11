"use client";

import { Plus, X } from "lucide-react";
import { useState } from "react";
import { TOKYO_23_WARDS } from "@/lib/filter-constants";
import { GRADIENT } from "@/lib/theme-tokens";

interface CityMultiSelectProps {
  selected: string[];
  onChange: (cities: string[]) => void;
}

export function CityMultiSelect({ selected, onChange }: CityMultiSelectProps) {
  const [open, setOpen] = useState(false);
  const available = TOKYO_23_WARDS.filter((c) => !selected.includes(c));

  const remove = (c: string) => {
    onChange(selected.filter((x) => x !== c));
  };

  const add = (c: string) => {
    onChange([...selected, c]);
    setOpen(false);
  };

  return (
    <div>
      <div className="mb-1.5 flex flex-wrap gap-1.5">
        {selected.map((city) => (
          <span
            key={city}
            className="inline-flex items-center gap-1 rounded-full px-2.5 py-1 text-[10px] font-semibold"
            style={{
              background: GRADIENT.brandTint,
              border: "1px solid rgba(99, 102, 241, 0.25)",
              color: "var(--brand-indigo)",
            }}
          >
            {city}
            <button
              type="button"
              onClick={() => remove(city)}
              aria-label={`Remove ${city}`}
              className="opacity-70 hover:opacity-100"
            >
              <X size={10} />
            </button>
          </span>
        ))}
        <button
          type="button"
          onClick={() => setOpen((o) => !o)}
          aria-label="Add city"
          className="inline-flex items-center gap-1 rounded-full px-2.5 py-1 text-[10px]"
          style={{
            background: "var(--neutral-100)",
            border: "1px dashed var(--neutral-200)",
            color: "var(--neutral-500)",
          }}
        >
          <Plus size={10} /> add
        </button>
      </div>

      {open && (
        <div
          className="rounded-[10px] p-1.5"
          style={{
            background: "var(--card-fill-solid)",
            border: "1px solid var(--neutral-200)",
            boxShadow: "var(--shadow-card-subtle)",
          }}
        >
          <div className="grid max-h-40 grid-cols-3 gap-1 overflow-y-auto">
            {available.map((c) => (
              <button
                key={c}
                type="button"
                onClick={() => add(c)}
                className="rounded-md px-2 py-1 text-left text-[10px] hover:bg-[var(--neutral-50)]"
                style={{ color: "var(--neutral-700)" }}
              >
                {c}
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

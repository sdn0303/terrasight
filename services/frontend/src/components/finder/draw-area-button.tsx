"use client";

import { Pencil } from "lucide-react";

export function DrawAreaButton() {
  return (
    <button
      type="button"
      disabled
      aria-label="Draw custom area (coming soon)"
      className="w-full rounded-[10px] px-3 py-2 text-[10px] font-semibold"
      style={{
        background: "var(--neutral-50)",
        border: "1px dashed var(--neutral-200)",
        color: "var(--neutral-500)",
        cursor: "not-allowed",
      }}
    >
      <Pencil size={11} className="mr-1 inline-block" aria-hidden="true" />
      カスタム範囲を描画 (Coming soon)
    </button>
  );
}

"use client";

import { ModeTabs } from "./mode-tabs";
import { LocaleToggle } from "./locale-toggle";

export function TopBar() {
  return (
    <header
      className="fixed top-0 left-0 right-0 z-50 flex items-center justify-between px-4 gap-4 h-12 bg-neutral-900 border-b border-neutral-800"
    >
      <ModeTabs />
      <div className="flex-1" />
      <div className="flex items-center gap-2">
        <LocaleToggle />
      </div>
    </header>
  );
}

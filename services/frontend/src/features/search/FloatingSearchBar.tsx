import { Search } from "lucide-react";
import { useState } from "react";

export function FloatingSearchBar() {
  const [query, setQuery] = useState("");

  return (
    <div
      className="fixed flex items-center gap-2 px-3"
      style={{
        right: "var(--ts-gap-panel)",
        top: "var(--ts-gap-panel)",
        height: "var(--ts-tab-height)",
        background: "var(--ts-bg-panel)",
        borderRadius: "var(--ts-panel-radius)",
        zIndex: 10,
        minWidth: 200,
      }}
    >
      <Search size={14} style={{ color: "var(--ts-text-muted)" }} />
      <input
        type="text"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder="住所・駅名を検索"
        className="bg-transparent border-none outline-none text-xs flex-1"
        style={{
          color: "var(--ts-text-primary)",
          fontFamily: "var(--font-sans)",
        }}
        aria-label="住所・駅名を検索"
      />
    </div>
  );
}

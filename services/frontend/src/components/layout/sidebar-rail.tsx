import {
  Layers as LayersIcon,
  Map as MapIcon,
  SearchCheck,
  Settings as SettingsIcon,
  Sparkles,
  Table2,
} from "lucide-react";
import type { ComponentType, SVGProps } from "react";
import { useNavigationLevel } from "@/hooks/use-navigation-level";
import { CARD_RADIUS, PAGE_INSET, RAIL_WIDTH } from "@/lib/layout";
import { GLOW_SHADOW, GRADIENT } from "@/lib/theme-tokens";
import { type LeftPanelKind, useUIStore } from "@/stores/ui-store";

type LucideIcon = ComponentType<SVGProps<SVGSVGElement> & { size?: number }>;

interface RailToolSpec {
  id: "map" | "finder" | "opportunities" | "layers" | "themes";
  label: string;
  icon: LucideIcon;
}

const TOOLS: RailToolSpec[] = [
  { id: "map", label: "Map", icon: MapIcon },
  { id: "finder", label: "Finder", icon: SearchCheck },
  { id: "opportunities", label: "Opportunities", icon: Table2 },
  { id: "layers", label: "Layers", icon: LayersIcon },
  { id: "themes", label: "Themes", icon: Sparkles },
];

const LEFT_PANEL_TOOLS: readonly LeftPanelKind[] = [
  "finder",
  "layers",
  "themes",
];

export function SidebarRail() {
  const level = useNavigationLevel();
  const leftPanel = useUIStore((s) => s.leftPanel);
  const bottomSheet = useUIStore((s) => s.bottomSheet);
  const insight = useUIStore((s) => s.insight);
  const setLeftPanel = useUIStore((s) => s.setLeftPanel);
  const toggleLeftPanel = useUIStore((s) => s.toggleLeftPanel);
  const setBottomSheet = useUIStore((s) => s.setBottomSheet);
  const setInsight = useUIStore((s) => s.setInsight);
  const setSettingsOpen = useUIStore((s) => s.setSettingsOpen);

  const isToolActive = (id: RailToolSpec["id"]): boolean => {
    if (id === "map") {
      // "Map" is a neutral home state — active only when every Layer 2
      // overlay (left panel, bottom sheet, insight drawer) is closed.
      return leftPanel === null && bottomSheet === null && insight === null;
    }
    if (id === "opportunities") {
      return bottomSheet === "opportunities";
    }
    return leftPanel === id;
  };

  const handleToolClick = (id: RailToolSpec["id"]) => {
    if (id === "map") {
      setLeftPanel(null);
      setBottomSheet(null);
      setInsight(null);
      return;
    }
    if (id === "opportunities") {
      setBottomSheet(bottomSheet === "opportunities" ? null : "opportunities");
      return;
    }
    if (LEFT_PANEL_TOOLS.includes(id as LeftPanelKind)) {
      toggleLeftPanel(id as LeftPanelKind);
    }
  };

  return (
    <aside
      aria-label="Primary navigation"
      className="absolute flex flex-col items-center py-[18px]"
      style={{
        top: PAGE_INSET,
        left: PAGE_INSET,
        bottom: PAGE_INSET,
        width: RAIL_WIDTH,
        background: "var(--card-fill)",
        borderRadius: CARD_RADIUS.rail,
        boxShadow: "var(--shadow-card-medium)",
        backdropFilter: "blur(24px)",
        zIndex: 30,
      }}
    >
      {/* Brand mark */}
      <figure
        aria-label="Terrasight"
        className="mb-5 flex h-11 w-11 items-center justify-center text-xs font-extrabold text-white"
        style={{
          background: GRADIENT.brand,
          borderRadius: 14,
          boxShadow: GLOW_SHADOW.primarySubtle,
        }}
      >
        TS
      </figure>

      {/* Tools */}
      <nav className="flex flex-col items-center gap-1.5">
        {(level === "L1" || level === "L2"
          ? TOOLS.filter((t) => t.id === "map" || t.id === "layers")
          : TOOLS
        ).map((tool) => (
          <RailTool
            key={tool.id}
            tool={tool}
            active={isToolActive(tool.id)}
            onClick={() => handleToolClick(tool.id)}
          />
        ))}
      </nav>

      {/* Spacer */}
      <div className="flex-1" />

      {/* Footer: settings + avatar */}
      <button
        type="button"
        onClick={() => setSettingsOpen(true)}
        aria-label="Settings"
        className="flex h-11 w-11 items-center justify-center text-[color:var(--neutral-400)] hover:text-[color:var(--neutral-600)]"
      >
        <SettingsIcon size={15} />
      </button>
      <span
        role="img"
        aria-label="User avatar"
        className="mt-2 flex h-9 w-9 items-center justify-center text-xs font-bold text-white"
        style={{
          background: GRADIENT.primary,
          borderRadius: 12,
          boxShadow: GLOW_SHADOW.primarySubtle,
        }}
      >
        T
      </span>
    </aside>
  );
}

interface RailToolProps {
  tool: RailToolSpec;
  active: boolean;
  onClick: () => void;
}

function RailTool({ tool, active, onClick }: RailToolProps) {
  const Icon = tool.icon;
  return (
    <button
      type="button"
      onClick={onClick}
      aria-label={tool.label}
      aria-pressed={active}
      className="flex flex-col items-center justify-center transition-colors"
      style={{
        width: 52,
        height: 52,
        borderRadius: 14,
        background: active ? GRADIENT.brandTint : "transparent",
        boxShadow: active
          ? "inset 0 0 0 1px rgba(99, 102, 241, 0.22)"
          : undefined,
        color: active ? "var(--brand-indigo)" : "var(--neutral-400)",
      }}
    >
      <Icon size={17} aria-hidden="true" />
      <span
        className="mt-0.5 text-[8px] font-semibold"
        style={{ color: active ? "var(--brand-indigo)" : "var(--neutral-500)" }}
      >
        {tool.label}
      </span>
    </button>
  );
}

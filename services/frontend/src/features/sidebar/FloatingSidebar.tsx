import {
  BarChart3,
  Compass,
  Layers,
  MapPin,
  Receipt,
  Settings,
  ShieldAlert,
  Users,
} from "lucide-react";
import type { TabId } from "@/features/tabs/tab-configs";
import { useUIStore } from "@/stores/ui-store";

interface SidebarItem {
  icon: React.ElementType;
  label: string;
  tab?: TabId;
  action?: "settings" | "infra";
}

const SIDEBAR_ITEMS: SidebarItem[] = [
  { icon: Compass, label: "探索", tab: "overview" },
  { icon: BarChart3, label: "スコア分析", tab: "overview" },
  { icon: Receipt, label: "取引", tab: "transactions" },
  { icon: ShieldAlert, label: "ハザード", tab: "hazard" },
  { icon: Layers, label: "地盤", tab: "ground" },
  { icon: MapPin, label: "インフラ", action: "infra" },
  { icon: Users, label: "人口", tab: "population" },
];

const BOTTOM_ITEMS: SidebarItem[] = [
  { icon: Settings, label: "設定", action: "settings" },
];

function SidebarIcon({
  item,
  isActive,
  onClick,
}: {
  item: SidebarItem;
  isActive: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex items-center justify-center w-11 h-11 rounded-lg transition-colors"
      style={{
        background: isActive ? "var(--ts-bg-tab-active)" : "transparent",
        color: isActive ? "var(--ts-accent)" : "var(--ts-text-muted)",
      }}
      title={item.label}
      aria-label={item.label}
      aria-current={isActive ? "page" : undefined}
    >
      <item.icon size={20} />
    </button>
  );
}

export function FloatingSidebar() {
  const activeTab = useUIStore((s) => s.activeTab);
  const setActiveTab = useUIStore((s) => s.setActiveTab);
  const setSidebarSection = useUIStore((s) => s.setSidebarSection);

  return (
    <nav
      className="fixed flex flex-col items-center justify-between py-3 gap-1"
      style={{
        left: "var(--ts-gap-panel)",
        top: "var(--ts-gap-panel)",
        width: "var(--ts-sidebar-width)",
        height: "calc(100vh - var(--ts-gap-panel) * 2)",
        background: "var(--ts-bg-sidebar)",
        borderRadius: "var(--ts-panel-radius)",
        zIndex: 10,
      }}
      aria-label="Main navigation"
    >
      {/* Logo */}
      <div
        className="flex items-center justify-center w-11 h-11 rounded-lg font-bold text-lg"
        style={{ color: "var(--ts-accent)" }}
      >
        T
      </div>

      {/* Main nav items */}
      <div className="flex flex-col items-center gap-1 flex-1 mt-2">
        {SIDEBAR_ITEMS.map((item) => (
          <SidebarIcon
            key={item.label}
            item={item}
            isActive={item.tab != null && activeTab === item.tab}
            onClick={() => {
              if (item.tab) setActiveTab(item.tab);
              if (item.action === "infra") setSidebarSection("explore");
            }}
          />
        ))}
      </div>

      {/* Bottom items */}
      <div className="flex flex-col items-center gap-1">
        {BOTTOM_ITEMS.map((item) => (
          <SidebarIcon
            key={item.label}
            item={item}
            isActive={false}
            onClick={() => {
              if (item.action === "settings") setSidebarSection("settings");
            }}
          />
        ))}
      </div>
    </nav>
  );
}

import { ChevronDown, ChevronUp } from "lucide-react";
import type { TabId } from "@/features/tabs/tab-configs";
import { useUIStore } from "@/stores/ui-store";

interface LegendEntry {
  color: string;
  label: string;
}

const LEGEND_DATA: Partial<
  Record<TabId, { title: string; entries: LegendEntry[] }>
> = {
  overview: {
    title: "TLS総合スコア",
    entries: [
      { color: "#10B981", label: "80-100  優良" },
      { color: "#3B82F6", label: "60-80   良好" },
      { color: "#FBBF24", label: "40-60   平均" },
      { color: "#F97316", label: "20-40   注意" },
      { color: "#EF4444", label: "0-20    危険" },
    ],
  },
  "land-price": {
    title: "公示地価 (万円/m²)",
    entries: [
      { color: "#7C3AED", label: ">300    高額" },
      { color: "#DC2626", label: "100-300" },
      { color: "#F97316", label: "50-100" },
      { color: "#FBBF24", label: "30-50" },
      { color: "#22C55E", label: "10-30" },
      { color: "#3B82F6", label: "<10     低" },
    ],
  },
  vacancy: {
    title: "空室率 (%)",
    entries: [
      { color: "#22C55E", label: "<5%     健全" },
      { color: "#3B82F6", label: "5-10%" },
      { color: "#FBBF24", label: "10-15%" },
      { color: "#EF4444", label: ">15%    危険" },
    ],
  },
  hazard: {
    title: "ハザードマップ",
    entries: [
      { color: "#3B82F6", label: "洪水浸水" },
      { color: "#EF4444", label: "液状化" },
      { color: "#7C3AED", label: "断層" },
      { color: "#F97316", label: "急傾斜" },
      { color: "#8B5CF6", label: "地震動" },
    ],
  },
  ground: {
    title: "地形分類",
    entries: [
      { color: "#92400E", label: "台地" },
      { color: "#D97706", label: "段丘" },
      { color: "#60A5FA", label: "低地" },
      { color: "#4ADE80", label: "丘陵" },
      { color: "#A3E635", label: "山地" },
    ],
  },
  zoning: {
    title: "用途地域",
    entries: [
      { color: "#22C55E", label: "住居系" },
      { color: "#EF4444", label: "商業系" },
      { color: "#F97316", label: "近隣商業" },
      { color: "#3B82F6", label: "準工業" },
      { color: "#A855F7", label: "工業系" },
    ],
  },
};

export function FloatingLegend() {
  const activeTab = useUIStore((s) => s.activeTab);
  const collapsed = useUIStore((s) => s.legendCollapsed);
  const toggle = useUIStore((s) => s.toggleLegend);

  const legend = LEGEND_DATA[activeTab];
  if (!legend) return null;

  return (
    <div
      className="fixed"
      style={{
        right: "var(--ts-gap-panel)",
        bottom: "var(--ts-gap-panel)",
        width: 196,
        background: "var(--ts-bg-panel)",
        borderRadius: "var(--ts-panel-radius)",
        zIndex: 10,
        padding: "var(--ts-panel-padding)",
      }}
    >
      <button
        type="button"
        onClick={toggle}
        className="flex items-center justify-between w-full text-xs font-medium"
        style={{ color: "var(--ts-text-secondary)" }}
      >
        {legend.title}
        {collapsed ? <ChevronUp size={14} /> : <ChevronDown size={14} />}
      </button>

      {!collapsed && (
        <div className="mt-2 flex flex-col gap-1.5">
          {legend.entries.map((entry) => (
            <div
              key={entry.label}
              className="flex items-center gap-2 text-xs"
              style={{ color: "var(--ts-text-secondary)" }}
            >
              <span
                className="inline-block w-3 h-3 rounded-sm shrink-0"
                style={{ background: entry.color }}
              />
              <span className="font-mono">{entry.label}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

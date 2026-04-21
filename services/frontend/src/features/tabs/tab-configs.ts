import {
  AlertTriangle,
  CircleDot,
  FileText,
  Globe,
  Grid3x3,
  Home,
  type LucideIcon,
  MapIcon,
  Mountain,
  Percent,
  Users,
} from "lucide-react";

export type TabId =
  | "overview"
  | "land-price"
  | "transactions"
  | "population"
  | "vacancy"
  | "stations"
  | "yield"
  | "hazard"
  | "ground"
  | "zoning";

export interface TabDefinition {
  id: TabId;
  label: string;
  icon: LucideIcon;
}

export const TABS: TabDefinition[] = [
  { id: "overview", label: "総合", icon: Globe },
  { id: "land-price", label: "地価", icon: MapIcon },
  { id: "transactions", label: "取引事例", icon: FileText },
  { id: "population", label: "人口・世帯", icon: Users },
  { id: "vacancy", label: "空室率", icon: Home },
  { id: "stations", label: "乗降客数", icon: CircleDot },
  { id: "yield", label: "利回り", icon: Percent },
  { id: "hazard", label: "ハザード", icon: AlertTriangle },
  { id: "ground", label: "地盤", icon: Mountain },
  { id: "zoning", label: "用途地域", icon: Grid3x3 },
];

export const DEFAULT_TAB: TabId = "overview";

export function isValidTabId(value: string): value is TabId {
  return TABS.some((t) => t.id === value);
}

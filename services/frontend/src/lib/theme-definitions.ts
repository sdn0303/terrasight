import {
  BarChart3,
  Building2,
  Droplets,
  type LucideIcon,
  MapPin,
  Train,
} from "lucide-react";

export type ThemeId =
  | "land-price"
  | "hazard"
  | "transactions"
  | "station"
  | "score";

export type SidebarCategory = "explore" | "view";

export interface ThemeDefinition {
  id: ThemeId;
  label: string;
  icon: LucideIcon;
  category: SidebarCategory;
}

export const THEMES: ThemeDefinition[] = [
  { id: "score", label: "スコア分析", icon: BarChart3, category: "explore" },
  { id: "land-price", label: "地価", icon: MapPin, category: "view" },
  {
    id: "transactions",
    label: "取引事例",
    icon: Building2,
    category: "view",
  },
  { id: "hazard", label: "ハザード", icon: Droplets, category: "view" },
  { id: "station", label: "乗降客数", icon: Train, category: "view" },
];

export function isValidThemeId(value: string): value is ThemeId {
  return THEMES.some((t) => t.id === value);
}

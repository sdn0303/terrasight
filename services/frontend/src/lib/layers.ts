export interface LayerConfig {
  id: string;
  name: string;
  nameJa: string;
  category: "pricing" | "urban" | "disaster" | "facilities";
  defaultEnabled: boolean;
}

export const LAYERS: LayerConfig[] = [
  {
    id: "landprice",
    name: "Land Price",
    nameJa: "地価公示",
    category: "pricing",
    defaultEnabled: true,
  },
  {
    id: "zoning",
    name: "Zoning",
    nameJa: "用途地域",
    category: "urban",
    defaultEnabled: true,
  },
  {
    id: "flood",
    name: "Flood Risk",
    nameJa: "洪水浸水",
    category: "disaster",
    defaultEnabled: false,
  },
  {
    id: "steep_slope",
    name: "Steep Slope",
    nameJa: "急傾斜地",
    category: "disaster",
    defaultEnabled: false,
  },
  {
    id: "schools",
    name: "Schools",
    nameJa: "学校",
    category: "facilities",
    defaultEnabled: false,
  },
  {
    id: "medical",
    name: "Medical",
    nameJa: "医療機関",
    category: "facilities",
    defaultEnabled: false,
  },
];

export const CATEGORIES = [
  { id: "pricing", label: "PRICING" },
  { id: "urban", label: "URBAN PLANNING" },
  { id: "disaster", label: "DISASTER RISK" },
  { id: "facilities", label: "FACILITIES" },
] as const;

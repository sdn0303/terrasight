export interface LayerConfig {
  id: string;
  name: string;
  nameJa: string;
  category: "pricing" | "urban" | "disaster" | "facilities" | "terrain";
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
  {
    id: "did",
    name: "DID Area",
    nameJa: "人口集中地区",
    category: "urban",
    defaultEnabled: false,
  },
  {
    id: "landform",
    name: "Landform",
    nameJa: "地形分類",
    category: "terrain",
    defaultEnabled: false,
  },
  {
    id: "geology",
    name: "Geology",
    nameJa: "表層地質",
    category: "terrain",
    defaultEnabled: false,
  },
  {
    id: "admin_boundary",
    name: "Admin Boundary",
    nameJa: "市町村境界",
    category: "urban",
    defaultEnabled: false,
  },
  {
    id: "fault",
    name: "Fault Lines",
    nameJa: "断層線",
    category: "disaster",
    defaultEnabled: false,
  },
  {
    id: "flood_history",
    name: "Flood History",
    nameJa: "浸水履歴",
    category: "disaster",
    defaultEnabled: false,
  },
  {
    id: "volcano",
    name: "Volcanoes",
    nameJa: "火山",
    category: "disaster",
    defaultEnabled: false,
  },
  {
    id: "soil",
    name: "Soil",
    nameJa: "土壌図",
    category: "terrain",
    defaultEnabled: false,
  },
];

export const CATEGORIES = [
  { id: "pricing", label: "PRICING" },
  { id: "urban", label: "URBAN PLANNING" },
  { id: "disaster", label: "DISASTER RISK" },
  { id: "facilities", label: "FACILITIES" },
  { id: "terrain", label: "TERRAIN" },
] as const;

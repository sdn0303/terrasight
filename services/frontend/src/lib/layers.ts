export interface LayerConfig {
  id: string;
  name: string;
  nameJa: string;
  category: "value" | "risk" | "ground" | "infra" | "orientation";
  defaultEnabled: boolean;
  /** CSS color token for the layer indicator dot */
  color: string;
}

export const LAYERS: LayerConfig[] = [
  {
    id: "landprice",
    name: "Land Price",
    nameJa: "地価公示",
    category: "value",
    defaultEnabled: true,
    color: "var(--layer-landprice)",
  },
  {
    id: "flood_history",
    name: "Flood History",
    nameJa: "浸水履歴",
    category: "value",
    defaultEnabled: false,
    color: "var(--layer-flood-history)",
  },
  {
    id: "did",
    name: "DID Area",
    nameJa: "人口集中地区",
    category: "value",
    defaultEnabled: false,
    color: "var(--layer-did)",
  },
  {
    id: "flood",
    name: "Flood Risk",
    nameJa: "洪水浸水",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-flood)",
  },
  {
    id: "steep_slope",
    name: "Steep Slope",
    nameJa: "急傾斜地",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-steep-slope)",
  },
  {
    id: "fault",
    name: "Fault Lines",
    nameJa: "断層線",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-fault)",
  },
  {
    id: "volcano",
    name: "Volcanoes",
    nameJa: "火山",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-volcano)",
  },
  {
    id: "landform",
    name: "Landform",
    nameJa: "地形分類",
    category: "ground",
    defaultEnabled: false,
    color: "var(--layer-landform)",
  },
  {
    id: "geology",
    name: "Geology",
    nameJa: "表層地質",
    category: "ground",
    defaultEnabled: false,
    color: "var(--layer-geology)",
  },
  {
    id: "soil",
    name: "Soil",
    nameJa: "土壌図",
    category: "ground",
    defaultEnabled: false,
    color: "var(--layer-soil)",
  },
  {
    id: "schools",
    name: "Schools",
    nameJa: "学校",
    category: "infra",
    defaultEnabled: false,
    color: "var(--layer-schools)",
  },
  {
    id: "medical",
    name: "Medical",
    nameJa: "医療機関",
    category: "infra",
    defaultEnabled: false,
    color: "var(--layer-medical)",
  },
  {
    id: "admin_boundary",
    name: "Admin Boundary",
    nameJa: "市町村境界",
    category: "orientation",
    defaultEnabled: true,
    color: "var(--layer-boundary)",
  },
  {
    id: "zoning",
    name: "Zoning",
    nameJa: "用途地域",
    category: "orientation",
    defaultEnabled: true,
    color: "var(--layer-zoning)",
  },
];

export const CATEGORIES = [
  { id: "value", label: "HOW MUCH?", labelJa: "投資価値" },
  { id: "risk", label: "IS IT SAFE?", labelJa: "リスク評価" },
  { id: "ground", label: "WHAT'S THE GROUND?", labelJa: "地盤" },
  { id: "infra", label: "WHAT'S NEARBY?", labelJa: "インフラ" },
  { id: "orientation", label: "WHERE AM I?", labelJa: "オリエンテーション" },
] as const;

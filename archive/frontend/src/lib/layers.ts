export interface LayerDef {
  id: string;
  name: string;
  nameJa: string;
  endpoint: string;
  defaultOn: boolean;
  category: "price" | "zoning" | "disaster" | "facility";
}

export const LAYERS: LayerDef[] = [
  { id: "landprice", name: "Land Prices", nameJa: "地価公示", endpoint: "XPT002", defaultOn: true, category: "price" },
  { id: "zoning", name: "Use Zones", nameJa: "用途地域", endpoint: "XKT002", defaultOn: false, category: "zoning" },
  { id: "liquefaction", name: "Liquefaction", nameJa: "液状化", endpoint: "XKT025", defaultOn: false, category: "disaster" },
  { id: "flood", name: "Flood Risk", nameJa: "洪水浸水", endpoint: "XKT026", defaultOn: false, category: "disaster" },
  { id: "steep_slope", name: "Steep Slope", nameJa: "急傾斜地", endpoint: "XKT022", defaultOn: false, category: "disaster" },
  { id: "schools", name: "Schools", nameJa: "学校", endpoint: "XKT006", defaultOn: false, category: "facility" },
  { id: "medical", name: "Medical", nameJa: "医療機関", endpoint: "XKT010", defaultOn: false, category: "facility" },
];

export type ActiveLayers = Record<string, boolean>;

export function getDefaultActiveLayers(): ActiveLayers {
  return Object.fromEntries(LAYERS.map((l) => [l.id, l.defaultOn]));
}

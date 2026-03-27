/** Field definition for click-inspect popup rendering */
export interface PopupField {
  key: string;
  label: string;
  /** Optional suffix appended after the value (e.g., "円/㎡", "人") */
  suffix?: string;
}

export interface LayerConfig {
  id: string;
  name: string;
  nameJa: string;
  category: "value" | "risk" | "ground" | "infra" | "orientation";
  defaultEnabled: boolean;
  /** CSS color token for the layer indicator dot */
  color: string;
  /**
   * Data source:
   * - 'api' layers receive FeatureCollection from useAreaData
   * - 'static' layers load from /geojson/ on mount
   * - 'timeseries' layers have their own dedicated TanStack Query hooks (e.g. useLandPrices)
   */
  source: "api" | "static" | "timeseries";
  /** Fields displayed in the PopupCard on click-inspect */
  popupFields?: PopupField[];
  /** MapLibre layer IDs that respond to click events (for interactiveLayerIds) */
  interactiveLayerIds?: string[];
  /** Minimum zoom level for this layer to be visible */
  minZoom?: number;
  /** Theme(s) this layer belongs to for automatic theme preset activation */
  theme?: Array<"safety" | "livability" | "price" | "future">;
}

export const LAYERS: LayerConfig[] = [
  {
    id: "land_price_ts",
    name: "Land Price (Time Series)",
    nameJa: "地価公示（時系列）",
    category: "value",
    defaultEnabled: true,
    color: "var(--layer-landprice)",
    source: "timeseries",
    popupFields: [
      { key: "address", label: "所在地" },
      { key: "price_per_sqm", label: "価格", suffix: "円/㎡" },
      { key: "year", label: "年度" },
      { key: "land_use", label: "用途" },
    ],
    interactiveLayerIds: ["land-price-extrusion-3d", "land-price-ts-circle"],
    theme: ["price"],
  },
  {
    id: "landprice",
    name: "Land Price",
    nameJa: "地価公示",
    category: "value",
    defaultEnabled: false,
    color: "var(--layer-landprice)",
    source: "api",
    popupFields: [
      { key: "address", label: "所在地" },
      { key: "price_per_sqm", label: "価格", suffix: "円/㎡" },
    ],
    interactiveLayerIds: ["landprice-circle"],
    theme: ["price"],
  },
  {
    id: "flood_history",
    name: "Flood History",
    nameJa: "浸水履歴",
    category: "value",
    defaultEnabled: false,
    color: "var(--layer-flood-history)",
    source: "static",
    popupFields: [
      { key: "name", label: "地域名" },
      { key: "depth", label: "浸水深", suffix: "m" },
      { key: "year", label: "発生年" },
    ],
    interactiveLayerIds: ["flood-history-fill"],
    theme: ["safety"],
  },
  {
    id: "did",
    name: "DID Area",
    nameJa: "人口集中地区",
    category: "value",
    defaultEnabled: false,
    color: "var(--layer-did)",
    source: "static",
    popupFields: [
      { key: "name", label: "地区名" },
      { key: "population", label: "人口", suffix: "人" },
    ],
    interactiveLayerIds: ["did-fill"],
    theme: ["future"],
  },
  {
    id: "station",
    name: "Railway Stations",
    nameJa: "鉄道駅",
    category: "value",
    defaultEnabled: false,
    color: "var(--layer-station)",
    source: "static",
    popupFields: [
      { key: "stationName", label: "駅名" },
      { key: "lineName", label: "路線名" },
      { key: "passengerCount", label: "乗降客数", suffix: "人/日" },
    ],
    interactiveLayerIds: ["station-circle"],
    theme: ["livability", "future"],
  },
  {
    id: "flood",
    name: "Flood Risk",
    nameJa: "洪水浸水",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-flood)",
    source: "api",
    popupFields: [
      { key: "river_name", label: "河川名" },
      { key: "depth_rank", label: "浸水深ランク" },
    ],
    interactiveLayerIds: ["flood-fill"],
    theme: ["safety"],
  },
  {
    id: "steep_slope",
    name: "Steep Slope",
    nameJa: "急傾斜地",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-steep-slope)",
    source: "api",
    popupFields: [{ key: "area_name", label: "区域名" }],
    interactiveLayerIds: ["steep-slope-fill"],
    theme: ["safety"],
  },
  {
    id: "liquefaction",
    name: "Liquefaction Risk",
    nameJa: "液状化危険度",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-liquefaction)",
    source: "static",
    popupFields: [{ key: "PL区分", label: "液状化危険度" }],
    interactiveLayerIds: ["liquefaction-circle"],
    minZoom: 11,
    theme: ["safety"],
  },
  {
    id: "seismic",
    name: "Seismic Hazard",
    nameJa: "地震動・震源断層",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-seismic)",
    source: "static",
    popupFields: [
      { key: "LTENAME", label: "断層名" },
      { key: "MAG", label: "マグニチュード" },
      { key: "AVR_T30P", label: "30年超過確率" },
      { key: "AVR_T50P", label: "50年超過確率" },
      { key: "LEN", label: "断層長", suffix: "km" },
    ],
    interactiveLayerIds: ["seismic-fill"],
    theme: ["safety"],
  },
  {
    id: "fault",
    name: "Fault Lines",
    nameJa: "断層線",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-fault)",
    source: "static",
    popupFields: [
      { key: "name", label: "断層名" },
      { key: "length", label: "延長", suffix: "km" },
    ],
    interactiveLayerIds: ["fault-line"],
    theme: ["safety"],
  },
  {
    id: "volcano",
    name: "Volcanoes",
    nameJa: "火山",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-volcano)",
    source: "static",
    popupFields: [
      { key: "name", label: "火山名" },
      { key: "rank", label: "ランク" },
    ],
    interactiveLayerIds: ["volcano-circle"],
    theme: ["safety"],
  },
  {
    id: "landslide",
    name: "Landslide Risk",
    nameJa: "土砂災害",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-landslide)",
    source: "static",
    popupFields: [
      { key: "areaName", label: "区域名" },
      { key: "riskType", label: "危険区分" },
      { key: "designation", label: "指定種別" },
    ],
    interactiveLayerIds: ["landslide-fill"],
    minZoom: 11,
    theme: ["safety"],
  },
  {
    id: "tsunami",
    name: "Tsunami Risk",
    nameJa: "津波浸水",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-tsunami)",
    source: "static",
    popupFields: [
      { key: "areaName", label: "地域名" },
      { key: "depth", label: "想定浸水深", suffix: "m" },
      { key: "scenario", label: "想定シナリオ" },
    ],
    interactiveLayerIds: ["tsunami-fill"],
    theme: ["safety"],
  },
  {
    id: "landform",
    name: "Landform",
    nameJa: "地形分類",
    category: "ground",
    defaultEnabled: false,
    color: "var(--layer-landform)",
    source: "static",
    popupFields: [{ key: "name", label: "地形区分" }],
    interactiveLayerIds: ["landform-fill"],
  },
  {
    id: "geology",
    name: "Geology",
    nameJa: "表層地質",
    category: "ground",
    defaultEnabled: false,
    color: "var(--layer-geology)",
    source: "static",
    popupFields: [{ key: "name", label: "地質名" }],
    interactiveLayerIds: ["geology-fill"],
  },
  {
    id: "soil",
    name: "Soil",
    nameJa: "土壌図",
    category: "ground",
    defaultEnabled: false,
    color: "var(--layer-soil)",
    source: "static",
    popupFields: [{ key: "soilCategory", label: "土壌分類" }],
    interactiveLayerIds: ["soil-fill"],
  },
  {
    id: "schools",
    name: "Schools",
    nameJa: "学校",
    category: "infra",
    defaultEnabled: false,
    color: "var(--layer-schools)",
    source: "api",
    popupFields: [
      { key: "name", label: "学校名" },
      { key: "school_type", label: "種別" },
    ],
    interactiveLayerIds: ["schools-circle"],
    theme: ["livability"],
  },
  {
    id: "medical",
    name: "Medical",
    nameJa: "医療機関",
    category: "infra",
    defaultEnabled: false,
    color: "var(--layer-medical)",
    source: "api",
    popupFields: [
      { key: "name", label: "施設名" },
      { key: "facility_type", label: "種別" },
      { key: "bed_count", label: "病床数" },
    ],
    interactiveLayerIds: ["medical-circle"],
    theme: ["livability"],
  },
  {
    id: "railway",
    name: "Railway Lines",
    nameJa: "鉄道路線",
    category: "infra",
    defaultEnabled: false,
    color: "var(--layer-railway)",
    source: "static",
    popupFields: [
      { key: "N02_003", label: "路線名" },
      { key: "N02_004", label: "事業者名" },
    ],
    interactiveLayerIds: ["railway-line"],
    theme: ["livability"],
  },
  {
    id: "admin_boundary",
    name: "Admin Boundary",
    nameJa: "市町村境界",
    category: "orientation",
    defaultEnabled: true,
    color: "var(--layer-boundary)",
    source: "static",
    popupFields: [{ key: "name", label: "市区町村名" }],
    interactiveLayerIds: ["admin-boundary-line"],
  },
  {
    id: "zoning",
    name: "Zoning",
    nameJa: "用途地域",
    category: "orientation",
    defaultEnabled: true,
    color: "var(--layer-zoning)",
    source: "api",
    popupFields: [
      { key: "zone_type", label: "用途地域名" },
      { key: "floor_area_ratio", label: "容積率", suffix: "%" },
      { key: "building_coverage", label: "建蔽率", suffix: "%" },
    ],
    interactiveLayerIds: ["zoning-fill"],
    theme: ["price", "future"],
  },
  {
    id: "population_mesh",
    name: "Population Mesh",
    nameJa: "将来人口メッシュ",
    category: "orientation",
    defaultEnabled: false,
    color: "var(--layer-population)",
    source: "static",
    popupFields: [
      { key: "meshCode", label: "メッシュコード" },
      { key: "pop2020", label: "2020年人口", suffix: "人" },
      { key: "pop2050", label: "2050年人口", suffix: "人" },
      { key: "changeRate", label: "増減率", suffix: "%" },
    ],
    interactiveLayerIds: ["population-mesh-fill"],
    minZoom: 13,
    theme: ["future"],
  },
];

export const CATEGORIES = [
  { id: "value", label: "HOW MUCH?", labelJa: "投資価値" },
  { id: "risk", label: "IS IT SAFE?", labelJa: "リスク評価" },
  { id: "ground", label: "WHAT'S THE GROUND?", labelJa: "地盤" },
  { id: "infra", label: "WHAT'S NEARBY?", labelJa: "インフラ" },
  { id: "orientation", label: "WHERE AM I?", labelJa: "オリエンテーション" },
] as const;

/** All interactive layer IDs derived from LAYERS config, used by MapView */
export const ALL_INTERACTIVE_LAYER_IDS: string[] = LAYERS.flatMap(
  (l) => l.interactiveLayerIds ?? [],
);

/** Get layers by source type */
export function getLayersBySource(
  source: "api" | "static" | "timeseries",
): LayerConfig[] {
  return LAYERS.filter((l) => l.source === source);
}

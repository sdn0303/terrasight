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
  /** Data source: 'api' layers receive FeatureCollection from useAreaData, 'static' layers load from /geojson/ */
  source: "api" | "static";
  /** Fields displayed in the PopupCard on click-inspect */
  popupFields?: PopupField[];
  /** MapLibre layer IDs that respond to click events (for interactiveLayerIds) */
  interactiveLayerIds?: string[];
  /** Minimum zoom level for this layer to be visible */
  minZoom?: number;
}

export const LAYERS: LayerConfig[] = [
  {
    id: "landprice",
    name: "Land Price",
    nameJa: "地価公示",
    category: "value",
    defaultEnabled: true,
    color: "var(--layer-landprice)",
    source: "api",
    popupFields: [
      { key: "address", label: "所在地" },
      { key: "price", label: "価格", suffix: "円/㎡" },
      { key: "change_rate", label: "変動率", suffix: "%" },
    ],
    interactiveLayerIds: ["landprice-circle"],
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
      { key: "name", label: "河川名" },
      { key: "depth", label: "想定浸水深", suffix: "m" },
    ],
    interactiveLayerIds: ["flood-fill"],
  },
  {
    id: "steep_slope",
    name: "Steep Slope",
    nameJa: "急傾斜地",
    category: "risk",
    defaultEnabled: false,
    color: "var(--layer-steep-slope)",
    source: "api",
    popupFields: [
      { key: "name", label: "区域名" },
      { key: "type", label: "種別" },
    ],
    interactiveLayerIds: ["steep-slope-fill"],
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
      { key: "type", label: "種別" },
    ],
    interactiveLayerIds: ["schools-circle"],
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
      { key: "type", label: "種別" },
    ],
    interactiveLayerIds: ["medical-circle"],
  },
  {
    id: "school_district",
    name: "School Districts",
    nameJa: "小学校区",
    category: "infra",
    defaultEnabled: false,
    color: "var(--layer-school-dist)",
    source: "static",
    popupFields: [
      { key: "schoolName", label: "学校名" },
      { key: "districtName", label: "学区名" },
    ],
    interactiveLayerIds: ["school-district-fill"],
    minZoom: 12,
  },
  {
    id: "park",
    name: "Parks",
    nameJa: "都市公園",
    category: "infra",
    defaultEnabled: false,
    color: "var(--layer-park)",
    source: "static",
    popupFields: [
      { key: "parkName", label: "公園名" },
      { key: "parkType", label: "種別" },
      { key: "area", label: "面積", suffix: "㎡" },
    ],
    interactiveLayerIds: ["park-fill"],
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
      { key: "name", label: "用途地域名" },
      { key: "far", label: "容積率", suffix: "%" },
      { key: "bcr", label: "建蔽率", suffix: "%" },
    ],
    interactiveLayerIds: ["zoning-fill"],
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
  },
  {
    id: "urban_plan",
    name: "Urban Planning Zones",
    nameJa: "立地適正化",
    category: "orientation",
    defaultEnabled: false,
    color: "var(--layer-urban-plan)",
    source: "static",
    popupFields: [
      { key: "zoneName", label: "区域名" },
      { key: "zoneType", label: "区域種別" },
    ],
    interactiveLayerIds: ["urban-plan-fill"],
    minZoom: 11,
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
export function getLayersBySource(source: "api" | "static"): LayerConfig[] {
  return LAYERS.filter((l) => l.source === source);
}

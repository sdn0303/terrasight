/**
 * 路線テーマカラーマッピング。
 * Key: `${operator_name}/${line_name}`
 * Ref: DESIGN.md Sec 7
 */
const RAILWAY_COLORS: Record<string, string> = {
  "JR東日本/山手線": "#9ACD32",
  "JR東日本/中央線快速": "#F15A22",
  "JR東日本/京浜東北線": "#00B2E5",
  "JR東日本/総武線各停": "#FFD400",
  "JR東日本/埼京線": "#00AC9B",
  "JR東日本/湘南新宿ライン": "#E76734",
  "JR東日本/常磐線快速": "#007642",
  "JR東日本/東海道本線": "#F68B1E",
  "東京メトロ/銀座線": "#FF9500",
  "東京メトロ/丸ノ内線": "#F62E36",
  "東京メトロ/日比谷線": "#B5B5AC",
  "東京メトロ/東西線": "#009BBF",
  "東京メトロ/千代田線": "#00A650",
  "東京メトロ/有楽町線": "#C1A470",
  "東京メトロ/半蔵門線": "#8B76D0",
  "東京メトロ/南北線": "#00ADA9",
  "東京メトロ/副都心線": "#9C5E31",
  "都営/浅草線": "#E85298",
  "都営/三田線": "#0079C2",
  "都営/新宿線": "#6CBB5A",
  "都営/大江戸線": "#B6007A",
  "東急/東横線": "#DA0442",
  "東急/田園都市線": "#22A63A",
  "東急/目黒線": "#009CD2",
  "小田急/小田原線": "#0078BA",
  "京王/京王線": "#C9167E",
  "京王/井の頭線": "#B0BF1A",
  "西武/池袋線": "#009CD2",
  "西武/新宿線": "#009CD2",
  "京急/本線": "#E5171F",
  "京成/本線": "#1E90FF",
  "つくばエクスプレス/常磐新線": "#2D5DA8",
};

const DEFAULT_COLOR = "#6B7280";

/**
 * 路線名からテーマカラーを取得。
 * マッチしない場合はデフォルト灰色。
 */
export function getRailwayColor(
  operatorName: string,
  lineName: string,
): string {
  return RAILWAY_COLORS[`${operatorName}/${lineName}`] ?? DEFAULT_COLOR;
}

export { DEFAULT_COLOR, RAILWAY_COLORS };

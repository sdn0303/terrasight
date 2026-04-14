const BASE_URL = process.env.NEXT_PUBLIC_DATA_URL ?? "/data/fgb";

export function layerUrl(prefCode: string, layer: string): string {
  return `${BASE_URL}/${prefCode}/${layer}.fgb`;
}

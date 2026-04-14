const BASE_URL = import.meta.env.VITE_DATA_URL ?? "/data/fgb";

export function layerUrl(prefCode: string, layer: string): string {
  return `${BASE_URL}/${prefCode}/${layer}.fgb`;
}

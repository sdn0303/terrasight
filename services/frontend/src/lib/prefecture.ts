import type { BBox } from "@/lib/api";

// ---------------------------------------------------------------------------
// Prefecture bounding boxes (approximate, [south, west, north, east])
// Derived from N03 administrative boundary data (MLIT)
// ---------------------------------------------------------------------------

export interface PrefBBox {
  code: string;
  south: number;
  west: number;
  north: number;
  east: number;
}

const PREFECTURE_BBOXES: readonly PrefBBox[] = [
  { code: "01", south: 41.34, west: 139.34, north: 45.56, east: 145.82 },
  { code: "02", south: 40.22, west: 139.5, north: 41.56, east: 141.68 },
  { code: "03", south: 38.75, west: 139.69, north: 40.45, east: 141.68 },
  { code: "04", south: 37.77, west: 140.27, north: 39.0, east: 141.68 },
  { code: "05", south: 39.0, west: 139.7, north: 40.51, east: 140.96 },
  { code: "06", south: 37.73, west: 139.52, north: 39.21, east: 140.64 },
  { code: "07", south: 36.79, west: 139.16, north: 37.97, east: 140.98 },
  { code: "08", south: 35.74, west: 139.69, north: 36.96, east: 140.85 },
  { code: "09", south: 36.2, west: 139.33, north: 37.15, east: 140.29 },
  { code: "10", south: 36.06, west: 138.64, north: 37.06, east: 139.68 },
  { code: "11", south: 35.77, west: 138.91, north: 36.28, east: 139.91 },
  { code: "12", south: 34.9, west: 139.77, north: 36.11, east: 140.87 },
  { code: "13", south: 35.5, west: 138.94, north: 35.9, east: 139.92 },
  { code: "14", south: 35.13, west: 138.91, north: 35.67, east: 139.78 },
  { code: "15", south: 37.0, west: 137.86, north: 38.55, east: 140.02 },
  { code: "16", south: 36.27, west: 136.77, north: 36.99, east: 137.76 },
  { code: "17", south: 36.07, west: 136.24, north: 37.85, east: 137.33 },
  { code: "18", south: 35.37, west: 135.45, north: 36.26, east: 136.82 },
  { code: "19", south: 35.18, west: 138.18, north: 35.97, east: 139.13 },
  { code: "20", south: 35.19, west: 137.32, north: 37.03, east: 138.73 },
  { code: "21", south: 35.14, west: 136.28, north: 36.48, east: 137.65 },
  { code: "22", south: 34.58, west: 137.49, north: 35.64, east: 139.17 },
  { code: "23", south: 34.57, west: 136.67, north: 35.42, east: 137.84 },
  { code: "24", south: 33.73, west: 135.85, north: 35.25, east: 137.0 },
  { code: "25", south: 34.77, west: 135.76, north: 35.57, east: 136.44 },
  { code: "26", south: 34.85, west: 135.44, north: 35.78, east: 136.06 },
  { code: "27", south: 34.27, west: 135.09, north: 34.82, east: 135.68 },
  { code: "28", south: 34.16, west: 134.26, north: 35.67, east: 135.47 },
  { code: "29", south: 33.85, west: 135.57, north: 34.75, east: 136.22 },
  { code: "30", south: 33.43, west: 135.07, north: 34.38, east: 136.01 },
  { code: "31", south: 35.08, west: 133.16, north: 35.62, east: 134.52 },
  { code: "32", south: 34.3, west: 131.66, north: 36.35, east: 133.38 },
  { code: "33", south: 34.36, west: 133.28, north: 35.33, east: 134.41 },
  { code: "34", south: 34.04, west: 132.05, north: 35.12, east: 133.48 },
  { code: "35", south: 33.74, west: 130.89, north: 34.77, east: 132.07 },
  { code: "36", south: 33.55, west: 133.55, north: 34.25, east: 134.81 },
  { code: "37", south: 34.05, west: 133.47, north: 34.52, east: 134.46 },
  { code: "38", south: 32.9, west: 132.01, north: 34.1, east: 133.69 },
  { code: "39", south: 32.71, west: 132.48, north: 33.88, east: 134.31 },
  { code: "40", south: 33.0, west: 130.02, north: 33.96, east: 131.19 },
  { code: "41", south: 32.97, west: 129.74, north: 33.6, east: 130.57 },
  { code: "42", south: 32.57, west: 128.61, north: 34.7, east: 130.34 },
  { code: "43", south: 32.07, west: 130.03, north: 33.19, east: 131.3 },
  { code: "44", south: 32.72, west: 130.83, north: 33.75, east: 132.13 },
  { code: "45", south: 31.36, west: 130.69, north: 32.88, east: 131.88 },
  { code: "46", south: 27.01, west: 128.56, north: 32.32, east: 131.19 },
  { code: "47", south: 24.05, west: 122.93, north: 27.89, east: 131.33 },
] as const;

// ---------------------------------------------------------------------------
// AABB intersection: two bboxes overlap when neither is entirely outside the other
// ---------------------------------------------------------------------------

function intersects(viewport: BBox, pref: PrefBBox): boolean {
  if (viewport.north < pref.south) return false;
  if (viewport.south > pref.north) return false;
  if (viewport.east < pref.west) return false;
  if (viewport.west > pref.east) return false;
  return true;
}

/**
 * Returns the bounding box for a given prefecture code, or undefined if not found.
 */
export function getBboxByCode(code: string): PrefBBox | undefined {
  return PREFECTURE_BBOXES.find((p) => p.code === code);
}

/**
 * Returns prefecture codes whose approximate bounding boxes intersect the
 * given viewport bbox.
 */
export function getPrefectureCodes(bbox: BBox): string[] {
  return PREFECTURE_BBOXES.filter((pref) => intersects(bbox, pref)).map(
    (pref) => pref.code,
  );
}

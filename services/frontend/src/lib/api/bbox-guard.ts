import type { BBox } from "@/lib/api";

const BBOX_MAX_SIDE = 0.5;

export function isBBoxValid(bbox: BBox): boolean {
  return (
    bbox.north - bbox.south <= BBOX_MAX_SIDE &&
    bbox.east - bbox.west <= BBOX_MAX_SIDE
  );
}

import ky from "ky";
import type { z } from "zod";
import {
  AreaDataResponse,
  HealthResponse,
  ScoreResponse,
  StatsResponse,
  TrendResponse,
} from "./schemas";

const api = ky.create({
  prefixUrl: process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8000",
  timeout: 10_000,
  retry: { limit: 1, statusCodes: [408, 429, 500, 502, 503, 504] },
});

async function get<T>(
  schema: z.ZodType<T>,
  path: string,
  params?: Record<string, string>,
): Promise<T> {
  const searchParams = params ? new URLSearchParams(params) : undefined;
  const data: unknown = await api.get(path, { searchParams }).json();
  return schema.parse(data);
}

// ─── Typed API functions ──────────────────────────────

export interface BBox {
  south: number;
  west: number;
  north: number;
  east: number;
}

export function fetchHealth() {
  return get(HealthResponse, "api/health");
}

export function fetchAreaData(bbox: BBox, layers: string[]) {
  return get(AreaDataResponse, "api/area-data", {
    south: String(bbox.south),
    west: String(bbox.west),
    north: String(bbox.north),
    east: String(bbox.east),
    layers: layers.join(","),
  });
}

export function fetchScore(lat: number, lng: number) {
  return get(ScoreResponse, "api/score", {
    lat: String(lat),
    lng: String(lng),
  });
}

export function fetchStats(bbox: BBox) {
  return get(StatsResponse, "api/stats", {
    south: String(bbox.south),
    west: String(bbox.west),
    north: String(bbox.north),
    east: String(bbox.east),
  });
}

export function fetchTrend(lat: number, lng: number, years?: number) {
  const params: Record<string, string> = {
    lat: String(lat),
    lng: String(lng),
  };
  if (years !== undefined) {
    params["years"] = String(years);
  }
  return get(TrendResponse, "api/trend", params);
}

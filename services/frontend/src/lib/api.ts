import ky from "ky";
import type { z } from "zod";
import { logger } from "./logger";
import {
  AreaDataResponse,
  HealthResponse,
  ScoreResponse,
  StatsResponse,
  TrendResponse,
} from "./schemas";

// ---------------------------------------------------------------------------
// ky instance with structured request/response logging
// ---------------------------------------------------------------------------

const log = logger.child({ module: "api" });

/**
 * Extracts a safe, loggable URL string (path + search, no origin secrets).
 * The prefixUrl is an env var and may be considered internal, so we log the
 * full URL — it contains no PII.
 */
function safeUrl(request: Request): string {
  try {
    const u = new URL(request.url);
    return `${u.pathname}${u.search}`;
  } catch {
    return request.url;
  }
}

const api = ky.create({
  prefixUrl: process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8000",
  timeout: 10_000,
  retry: { limit: 1, statusCodes: [408, 429, 500, 502, 503, 504] },
  hooks: {
    beforeRequest: [
      (request) => {
        log.debug(
          { method: request.method, url: safeUrl(request) },
          "api request",
        );
      },
    ],
    afterResponse: [
      (request, _options, response) => {
        const level = response.ok ? "debug" : "warn";
        log[level](
          {
            method: request.method,
            url: safeUrl(request),
            status: response.status,
          },
          "api response",
        );
        return response;
      },
    ],
    beforeError: [
      (error) => {
        log.error(
          {
            method: error.request.method,
            url: safeUrl(error.request),
            status: error.response?.status,
            err: error,
          },
          "api error",
        );
        return error;
      },
    ],
  },
});

// ---------------------------------------------------------------------------
// Generic typed fetch helper
// ---------------------------------------------------------------------------

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
    params.years = String(years);
  }
  return get(TrendResponse, "api/trend", params);
}

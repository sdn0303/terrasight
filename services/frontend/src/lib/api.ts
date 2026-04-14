import ky from "ky";
import type { z } from "zod";
import { logger } from "./logger";
import {
  AreaDataResponse,
  AreaStatsResponse,
  HealthResponse,
  LandPriceTimeSeriesResponse,
  OpportunitiesResponse,
  StatsResponse,
  TlsResponse,
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
  prefixUrl: import.meta.env.VITE_API_URL ?? "http://localhost:8000",
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
  signal?: AbortSignal,
): Promise<T> {
  const searchParams = params ? new URLSearchParams(params) : undefined;
  const data: unknown = await api
    .get(path, { searchParams, signal: signal ?? null })
    .json();
  return schema.parse(data);
}

// ─── Typed API functions ──────────────────────────────

export interface BBox {
  south: number;
  west: number;
  north: number;
  east: number;
}

export function fetchHealth(signal?: AbortSignal) {
  return get(HealthResponse, "api/health", undefined, signal);
}

export function fetchAreaData(
  bbox: BBox,
  layers: string[],
  zoom: number,
  signal?: AbortSignal,
) {
  return get(
    AreaDataResponse,
    "api/area-data",
    {
      south: String(bbox.south),
      west: String(bbox.west),
      north: String(bbox.north),
      east: String(bbox.east),
      layers: layers.join(","),
      zoom: String(Math.floor(zoom)),
    },
    signal,
  );
}

export function fetchScore(
  lat: number,
  lng: number,
  preset: string,
  signal?: AbortSignal,
) {
  return get(
    TlsResponse,
    "api/score",
    {
      lat: String(lat),
      lng: String(lng),
      preset,
    },
    signal,
  );
}

export function fetchStats(bbox: BBox, signal?: AbortSignal) {
  return get(
    StatsResponse,
    "api/stats",
    {
      south: String(bbox.south),
      west: String(bbox.west),
      north: String(bbox.north),
      east: String(bbox.east),
    },
    signal,
  );
}

export function fetchLandPrices(
  bbox: BBox,
  year: number,
  zoom: number,
  signal?: AbortSignal,
) {
  return get(
    LandPriceTimeSeriesResponse,
    "api/v1/land-prices",
    {
      year: String(year),
      bbox: `${bbox.west},${bbox.south},${bbox.east},${bbox.north}`,
      zoom: String(Math.floor(zoom)),
    },
    signal,
  );
}

export function fetchLandPricesAllYears(
  bbox: BBox,
  fromYear: number,
  toYear: number,
  zoom: number,
  signal?: AbortSignal,
) {
  return get(
    LandPriceTimeSeriesResponse,
    "api/v1/land-prices/all-years",
    {
      bbox: `${bbox.west},${bbox.south},${bbox.east},${bbox.north}`,
      from: String(fromYear),
      to: String(toYear),
      zoom: String(Math.floor(zoom)),
    },
    signal,
  );
}

export function fetchTrend(
  lat: number,
  lng: number,
  years?: number,
  signal?: AbortSignal,
) {
  const params: Record<string, string> = {
    lat: String(lat),
    lng: String(lng),
  };
  if (years !== undefined) {
    params.years = String(years);
  }
  return get(TrendResponse, "api/trend", params, signal);
}

export function fetchAreaStats(code: string, signal?: AbortSignal) {
  return get(AreaStatsResponse, "api/area-stats", { code }, signal);
}

// ---------------------------------------------------------------------------
// Opportunities (Phase 5)
// ---------------------------------------------------------------------------

export interface FetchOpportunitiesParams {
  bbox: BBox;
  limit?: number | undefined;
  offset?: number | undefined;
  tlsMin?: number | undefined;
  riskMax?: "low" | "mid" | "high" | undefined;
  zones?: string[] | undefined;
  stationMax?: number | undefined;
  priceMin?: number | undefined;
  priceMax?: number | undefined;
  preset?: string | undefined;
}

export function fetchOpportunities(
  params: FetchOpportunitiesParams,
  signal?: AbortSignal,
) {
  const query: Record<string, string> = {
    bbox: `${params.bbox.west},${params.bbox.south},${params.bbox.east},${params.bbox.north}`,
    limit: String(params.limit ?? 50),
    offset: String(params.offset ?? 0),
  };
  if (params.tlsMin !== undefined) query.tls_min = String(params.tlsMin);
  if (params.riskMax !== undefined) query.risk_max = params.riskMax;
  if (params.zones !== undefined && params.zones.length > 0) {
    query.zones = params.zones.join(",");
  }
  if (params.stationMax !== undefined)
    query.station_max = String(params.stationMax);
  if (params.priceMin !== undefined) query.price_min = String(params.priceMin);
  if (params.priceMax !== undefined) query.price_max = String(params.priceMax);
  if (params.preset !== undefined) query.preset = params.preset;

  return get(OpportunitiesResponse, "api/v1/opportunities", query, signal);
}

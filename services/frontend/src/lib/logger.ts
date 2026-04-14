/**
 * Singleton pino logger for the frontend application.
 *
 * - Server-side (Node.js / SSR): emits JSON in production, pino-pretty in dev
 * - Browser-side: pino's browser shim routes to console.*
 *
 * Usage:
 *   import { logger } from "@/lib/logger";
 *   logger.info({ userId: "123" }, "User signed in");
 *   logger.error({ err }, "Something went wrong");
 *
 * Child loggers (add a persistent context field):
 *   const log = logger.child({ module: "api" });
 */

import pino from "pino";

// ---------------------------------------------------------------------------
// Level resolution
// ---------------------------------------------------------------------------

const VALID_LEVELS = [
  "trace",
  "debug",
  "info",
  "warn",
  "error",
  "fatal",
] as const;
type LogLevel = (typeof VALID_LEVELS)[number];

function resolveLevel(): LogLevel {
  const raw = import.meta.env.VITE_LOG_LEVEL;
  if (raw !== undefined && (VALID_LEVELS as readonly string[]).includes(raw)) {
    return raw as LogLevel;
  }
  return "info";
}

// ---------------------------------------------------------------------------
// Version
// ---------------------------------------------------------------------------

const SERVICE_VERSION = import.meta.env.VITE_APP_VERSION ?? "unknown";

// ---------------------------------------------------------------------------
// Transport (Node.js only — pino-pretty for development)
// ---------------------------------------------------------------------------

function buildTransport(): pino.TransportSingleOptions | undefined {
  // Vite SPA is browser-only; pino-pretty is not needed.
  return undefined;
}

// ---------------------------------------------------------------------------
// Logger singleton
// ---------------------------------------------------------------------------

const transport = buildTransport();

export const logger = pino({
  level: resolveLevel(),
  browser: {
    // In the browser pino routes each level to the matching console method.
    // asObject: true would emit JSON strings; keep false for readable DevTools.
    asObject: false,
  },
  base: {
    service: "frontend",
    version: SERVICE_VERSION,
    // `pid` and `hostname` are Node.js-only; pino omits them in the browser
    // automatically via its browser shim.
  },
  ...(transport !== undefined ? { transport } : {}),
});

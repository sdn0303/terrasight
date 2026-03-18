"use client";

import { motion, AnimatePresence } from "framer-motion";

export interface ScoreCardData {
  // Location
  address?: string;

  // Price info
  pricePerSqm?: number;
  landPrice?: number;

  // Zoning
  zoneType?: string;

  // Disaster risk
  liquefactionRisk?: string;
  floodDepth?: number;

  // Facilities
  nearestSchool?: string;
  nearestMedical?: string;

  // Raw properties from any clicked feature
  properties?: Record<string, unknown>;
}

interface ScoreCardProps {
  data: ScoreCardData | null;
}

export default function ScoreCard({ data }: ScoreCardProps) {
  return (
    <AnimatePresence>
      {data && (
        <motion.div
          initial={{ x: 320, opacity: 0 }}
          animate={{ x: 0, opacity: 1 }}
          exit={{ x: 320, opacity: 0 }}
          transition={{ duration: 0.3, ease: "easeOut" }}
          className="fixed right-4 top-24 z-[200] w-80 rounded-lg border backdrop-blur-md overflow-hidden"
          style={{
            background: "rgba(10, 10, 15, 0.9)",
            borderColor: "var(--border-primary)",
          }}
        >
          {/* Header */}
          <div
            className="px-5 py-3 border-b"
            style={{ borderColor: "var(--border-primary)" }}
          >
            <span
              className="text-[10px] font-semibold tracking-widest"
              style={{ color: "var(--text-muted)" }}
            >
              PROPERTY INTEL
            </span>
          </div>

          <div className="p-4 flex flex-col gap-3 max-h-[60vh] overflow-y-auto styled-scrollbar">
            {/* Address */}
            {data.address && (
              <div>
                <div
                  className="text-[9px] tracking-widest mb-1"
                  style={{ color: "var(--text-muted)" }}
                >
                  LOCATION
                </div>
                <div
                  className="text-xs"
                  style={{ color: "var(--text-primary)" }}
                >
                  {data.address}
                </div>
              </div>
            )}

            {/* Price section */}
            {(data.pricePerSqm || data.landPrice) && (
              <div
                className="rounded-md p-3 border"
                style={{
                  borderColor: "var(--border-primary)",
                  background: "var(--bg-secondary)",
                }}
              >
                <div
                  className="text-[9px] tracking-widest mb-2"
                  style={{ color: "var(--accent-cyan)" }}
                >
                  PRICING
                </div>
                {data.pricePerSqm && (
                  <div className="flex justify-between items-baseline">
                    <span
                      className="text-[10px]"
                      style={{ color: "var(--text-muted)" }}
                    >
                      per sqm
                    </span>
                    <span
                      className="text-sm font-bold"
                      style={{ color: "var(--accent-cyan)" }}
                    >
                      {data.pricePerSqm.toLocaleString()}
                    </span>
                  </div>
                )}
                {data.landPrice && (
                  <div className="flex justify-between items-baseline mt-1">
                    <span
                      className="text-[10px]"
                      style={{ color: "var(--text-muted)" }}
                    >
                      land price
                    </span>
                    <span
                      className="text-sm font-bold"
                      style={{ color: "var(--text-primary)" }}
                    >
                      {data.landPrice.toLocaleString()}/sqm
                    </span>
                  </div>
                )}
              </div>
            )}

            {/* Zoning */}
            {data.zoneType && (
              <div>
                <div
                  className="text-[9px] tracking-widest mb-1"
                  style={{ color: "var(--accent-warning)" }}
                >
                  ZONING
                </div>
                <div
                  className="text-xs"
                  style={{ color: "var(--text-primary)" }}
                >
                  {data.zoneType}
                </div>
              </div>
            )}

            {/* Disaster Risk */}
            {(data.liquefactionRisk || data.floodDepth !== undefined) && (
              <div
                className="rounded-md p-3 border"
                style={{
                  borderColor: "rgba(224, 64, 48, 0.3)",
                  background: "rgba(224, 64, 48, 0.05)",
                }}
              >
                <div
                  className="text-[9px] tracking-widest mb-2"
                  style={{ color: "var(--accent-danger)" }}
                >
                  DISASTER RISK
                </div>
                {data.liquefactionRisk && (
                  <div className="flex justify-between items-baseline">
                    <span
                      className="text-[10px]"
                      style={{ color: "var(--text-muted)" }}
                    >
                      liquefaction
                    </span>
                    <span
                      className="text-xs font-bold"
                      style={{ color: "var(--accent-danger)" }}
                    >
                      {data.liquefactionRisk}
                    </span>
                  </div>
                )}
                {data.floodDepth !== undefined && (
                  <div className="flex justify-between items-baseline mt-1">
                    <span
                      className="text-[10px]"
                      style={{ color: "var(--text-muted)" }}
                    >
                      flood depth
                    </span>
                    <span
                      className="text-xs font-bold"
                      style={{ color: "var(--accent-danger)" }}
                    >
                      {data.floodDepth}m
                    </span>
                  </div>
                )}
              </div>
            )}

            {/* Facilities */}
            {(data.nearestSchool || data.nearestMedical) && (
              <div>
                <div
                  className="text-[9px] tracking-widest mb-1"
                  style={{ color: "#10b981" }}
                >
                  FACILITIES
                </div>
                {data.nearestSchool && (
                  <div
                    className="text-[10px] mt-1"
                    style={{ color: "var(--text-muted)" }}
                  >
                    [School] {data.nearestSchool}
                  </div>
                )}
                {data.nearestMedical && (
                  <div
                    className="text-[10px] mt-1"
                    style={{ color: "var(--text-muted)" }}
                  >
                    [Medical] {data.nearestMedical}
                  </div>
                )}
              </div>
            )}

            {/* Raw properties (debug/exploration) */}
            {data.properties && Object.keys(data.properties).length > 0 && (
              <div>
                <div
                  className="text-[9px] tracking-widest mb-1"
                  style={{ color: "var(--text-muted)" }}
                >
                  RAW DATA
                </div>
                <div
                  className="rounded-md p-2 text-[9px] font-mono overflow-x-auto styled-scrollbar"
                  style={{
                    background: "var(--bg-primary)",
                    color: "var(--text-muted)",
                  }}
                >
                  {Object.entries(data.properties)
                    .slice(0, 15)
                    .map(([k, v]) => (
                      <div key={k} className="flex gap-2 py-0.5">
                        <span style={{ color: "var(--accent-cyan)" }}>
                          {k}:
                        </span>
                        <span>{String(v)}</span>
                      </div>
                    ))}
                </div>
              </div>
            )}
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

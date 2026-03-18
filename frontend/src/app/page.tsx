"use client";

import dynamic from "next/dynamic";
import { useEffect, useState } from "react";
import { fetchAPI } from "@/lib/api";

const MapView = dynamic(() => import("@/components/MapView"), { ssr: false });

type HealthStatus = "loading" | "online" | "offline";

export default function Home() {
  const [status, setStatus] = useState<HealthStatus>("loading");

  useEffect(() => {
    fetchAPI<{ status: string }>("/api/health")
      .then(() => setStatus("online"))
      .catch(() => setStatus("offline"));
  }, []);

  return (
    <div style={{ width: "100vw", height: "100vh", position: "relative" }}>
      <MapView />

      {/* Header overlay */}
      <div
        style={{
          position: "absolute",
          top: 24,
          left: 24,
          zIndex: 10,
          pointerEvents: "none",
        }}
      >
        <h1
          style={{
            fontSize: "1.25rem",
            fontWeight: 700,
            letterSpacing: "0.15em",
            color: "var(--accent-cyan)",
            margin: 0,
            lineHeight: 1.2,
          }}
        >
          不動産投資 VISUALIZER
        </h1>
        <p
          style={{
            fontSize: "0.65rem",
            letterSpacing: "0.25em",
            color: "var(--text-muted)",
            margin: "4px 0 0 0",
          }}
        >
          MLIT GEOSPATIAL DATA PLATFORM
        </p>
      </div>

      {/* Status indicator */}
      <div
        style={{
          position: "absolute",
          bottom: 16,
          left: "50%",
          transform: "translateX(-50%)",
          zIndex: 10,
          display: "flex",
          alignItems: "center",
          gap: 8,
          padding: "6px 16px",
          background: "var(--bg-secondary)",
          border: "1px solid var(--border-primary)",
          borderRadius: 6,
          fontSize: "0.7rem",
          letterSpacing: "0.1em",
          color: "var(--text-secondary)",
        }}
      >
        <span
          style={{
            width: 6,
            height: 6,
            borderRadius: "50%",
            background:
              status === "online"
                ? "var(--accent-cyan)"
                : status === "offline"
                  ? "var(--accent-danger)"
                  : "var(--text-muted)",
          }}
        />
        BACKEND {status === "loading" ? "..." : status.toUpperCase()}
      </div>
    </div>
  );
}

"use client";

import { useEffect, useState } from "react";
import { spatialEngine } from "@/lib/wasm/spatial-engine";

export function useSpatialEngineReady(): boolean {
  const [ready, setReady] = useState(spatialEngine.ready);
  useEffect(() => spatialEngine.onReady(setReady), []);
  return ready;
}

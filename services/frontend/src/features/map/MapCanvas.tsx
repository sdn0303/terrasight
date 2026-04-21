import { Suspense, useCallback, useState } from "react";
import { MapLayers } from "@/components/map/map-layers";
import { MapView } from "@/components/map/map-view";
import { DetailPanel } from "@/features/detail/DetailPanel";
import { FloatingLegend } from "@/features/legend/FloatingLegend";
import { FloatingSearchBar } from "@/features/search/FloatingSearchBar";
import { FloatingSidebar } from "@/features/sidebar/FloatingSidebar";
import { FloatingTabBar } from "@/features/tabs/FloatingTabBar";
import type { BBox } from "@/lib/api";

export function MapCanvas() {
  const [bbox, setBbox] = useState<BBox | null>(null);
  const handleMoveEnd = useCallback((newBbox: BBox) => setBbox(newBbox), []);

  return (
    <div className="relative h-screen w-screen overflow-hidden">
      {/* Full-screen map (z-0) */}
      <div className="absolute inset-0">
        <MapView onMoveEnd={handleMoveEnd}>
          <Suspense fallback={null}>
            <MapLayers bbox={bbox} />
          </Suspense>
        </MapView>
      </div>

      {/* Floating UI components (z-10+) */}
      <FloatingSidebar />
      <FloatingTabBar />
      <FloatingSearchBar />
      <FloatingLegend />
      <DetailPanel />
    </div>
  );
}

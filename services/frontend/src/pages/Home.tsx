import { useCallback, useState } from "react";
import { AppShell } from "@/components/layout/app-shell";
import { MapLayers } from "@/components/map/map-layers";
import { MapView } from "@/components/map/map-view";
import { OpportunitiesTable } from "@/components/opportunities/opportunities-table";
import type { BBox } from "@/lib/api";

export default function Home() {
  const [bbox, setBbox] = useState<BBox | null>(null);
  const handleMoveEnd = useCallback((newBbox: BBox) => setBbox(newBbox), []);

  return (
    <AppShell>
      <MapView onMoveEnd={handleMoveEnd}>
        <MapLayers bbox={bbox} />
      </MapView>
      <OpportunitiesTable />
    </AppShell>
  );
}

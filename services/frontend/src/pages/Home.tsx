import { AppShell } from "@/components/layout/app-shell";
import { MapView } from "@/components/map/map-view";
import { OpportunitiesTable } from "@/components/opportunities/opportunities-table";
import { useMapUrlState } from "@/hooks/use-map-url-state";

export default function Home() {
  useMapUrlState();

  return (
    <AppShell>
      <MapView />
      <OpportunitiesTable />
    </AppShell>
  );
}

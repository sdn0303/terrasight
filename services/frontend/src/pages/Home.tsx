import { AppShell } from "@/components/layout/app-shell";
import { MapView } from "@/components/map/map-view";
import { useMapUrlState } from "@/hooks/use-map-url-state";

export default function Home() {
  useMapUrlState();

  return (
    <AppShell>
      <MapView />
    </AppShell>
  );
}

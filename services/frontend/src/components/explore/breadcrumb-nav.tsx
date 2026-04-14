"use client";

import { ChevronRight } from "lucide-react";
import { useNavigationLevel } from "@/hooks/use-navigation-level";
import { useMapStore } from "@/stores/map-store";
import { usePrefectureStore } from "@/stores/prefecture-store";
import { useUIStore } from "@/stores/ui-store";

/** Zoom level for the national (L1) view. */
const NATIONAL_ZOOM = 5;

function Separator() {
  return <ChevronRight size={12} className="text-ds-text-muted" aria-hidden />;
}

export function BreadcrumbNav() {
  const level = useNavigationLevel();

  const selectedPrefCode = usePrefectureStore((s) => s.selectedPrefCode);
  const selectedPrefName = usePrefectureStore((s) => s.selectedPrefName);

  const viewState = useMapStore((s) => s.viewState);
  const setViewState = useMapStore((s) => s.setViewState);
  const selectedArea = useMapStore((s) => s.selectedArea);
  const flyToPrefecture = useMapStore((s) => s.flyToPrefecture);

  const setInsight = useUIStore((s) => s.setInsight);

  function handleNational() {
    setViewState({ ...viewState, zoom: NATIONAL_ZOOM });
  }

  function handlePrefecture() {
    flyToPrefecture(selectedPrefCode);
    setInsight(null);
  }

  function handleMunicipality() {
    setInsight(null);
  }

  const isNationalClickable = level !== "L1";
  const isPrefClickable = level === "L3" || level === "L4";
  const isMuniClickable = level === "L4";

  return (
    <nav
      className="flex items-center gap-1.5 px-4 py-2 text-[10px] font-medium tracking-wide text-ds-text-muted"
      aria-label="Area breadcrumb"
    >
      {/* L1: 日本全国 */}
      <button
        type="button"
        onClick={handleNational}
        disabled={!isNationalClickable}
        className={
          isNationalClickable
            ? "text-ds-accent-primary hover:underline"
            : "text-ds-text-primary cursor-default"
        }
        aria-current={level === "L1" ? "page" : undefined}
      >
        日本全国
      </button>

      {/* L2: 都道府県名 */}
      {(level === "L2" || level === "L3" || level === "L4") && (
        <>
          <Separator />
          <button
            type="button"
            onClick={handlePrefecture}
            disabled={!isPrefClickable}
            className={
              isPrefClickable
                ? "text-ds-accent-primary hover:underline"
                : "text-ds-text-primary cursor-default"
            }
            aria-current={level === "L2" ? "page" : undefined}
          >
            {selectedPrefName}
          </button>
        </>
      )}

      {/* L3: 市区町村名 */}
      {(level === "L3" || level === "L4") && selectedArea !== null && (
        <>
          <Separator />
          <button
            type="button"
            onClick={handleMunicipality}
            disabled={!isMuniClickable}
            className={
              isMuniClickable
                ? "text-ds-accent-primary hover:underline"
                : "text-ds-text-primary cursor-default"
            }
            aria-current={level === "L3" ? "page" : undefined}
          >
            {selectedArea.name}
          </button>
        </>
      )}

      {/* L4: 物件詳細 */}
      {level === "L4" && (
        <>
          <Separator />
          <span className="text-ds-text-primary" aria-current="page">
            物件詳細
          </span>
        </>
      )}
    </nav>
  );
}

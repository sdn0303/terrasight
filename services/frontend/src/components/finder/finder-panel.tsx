"use client";

import { Search } from "lucide-react";
import { LeftPanel } from "@/components/layout/left-panel";
import { FilterSection } from "@/components/ui/filter-section";
import { GLOW_SHADOW, GRADIENT } from "@/lib/theme-tokens";
import { useFilterStore } from "@/stores/filter-store";
import { CityMultiSelect } from "./city-multi-select";
import { CriteriaSliders } from "./criteria-sliders";
import { DrawAreaButton } from "./draw-area-button";
import { PrefDropdown } from "./pref-dropdown";
import { PresetCardGrid } from "./preset-card-grid";
import { ZoneChips } from "./zone-chips";

interface FinderPanelProps {
  open: boolean;
  onClose: () => void;
  onSearch: () => void;
  matchCount: number;
}

export function FinderPanel({
  open,
  onClose,
  onSearch,
  matchCount,
}: FinderPanelProps) {
  const prefecture = useFilterStore((s) => s.area.prefecture);
  const cities = useFilterStore((s) => s.area.cities);
  const setArea = useFilterStore((s) => s.setArea);
  const formattedCount = matchCount.toLocaleString("en-US");

  return (
    <LeftPanel
      open={open}
      onClose={onClose}
      title="Investment Finder"
      subtitle={`${formattedCount} properties match`}
      badge="FINDER"
      footer={
        <div className="flex gap-2">
          <button
            type="button"
            aria-label="Save search"
            className="flex-1 rounded-[12px] py-2.5 text-[10px] font-bold"
            style={{
              background: "var(--neutral-100)",
              color: "var(--neutral-600)",
            }}
          >
            💾 Save
          </button>
          <button
            type="button"
            onClick={onSearch}
            aria-label={`${formattedCount} 物件を検索`}
            className="flex flex-[2] items-center justify-center gap-1.5 rounded-[12px] py-2.5 text-[10px] font-extrabold text-white"
            style={{
              background: GRADIENT.brand,
              boxShadow: GLOW_SHADOW.primary,
            }}
          >
            <Search size={12} aria-hidden="true" />
            {formattedCount} 物件を検索
          </button>
        </div>
      }
    >
      <FilterSection title="● AREA" required>
        <PrefDropdown value={prefecture} />
        <div className="h-1.5" />
        <CityMultiSelect
          selected={cities}
          onChange={(next) => setArea({ cities: next })}
        />
        <div className="h-1.5" />
        <DrawAreaButton />
      </FilterSection>

      <FilterSection title="● CRITERIA">
        <CriteriaSliders />
      </FilterSection>

      <FilterSection title="● ZONING & ACCESS">
        <ZoneChips />
        <div className="mt-3">
          <StationMaxDistanceSlider />
        </div>
      </FilterSection>

      <FilterSection title="● WEIGHT PRESET">
        <PresetCardGrid />
      </FilterSection>
    </LeftPanel>
  );
}

function StationMaxDistanceSlider() {
  const stationMax = useFilterStore((s) => s.zoning.stationMaxDistanceM);
  const setZoning = useFilterStore((s) => s.setZoning);
  return (
    <div>
      <div className="mb-1.5 flex items-center justify-between text-[10px]">
        <span style={{ color: "var(--neutral-600)" }}>駅からの距離</span>
        <span
          className="font-extrabold"
          style={{ color: "var(--neutral-900)" }}
        >
          ≤ {stationMax}m
        </span>
      </div>
      <input
        type="range"
        min={100}
        max={2000}
        step={50}
        value={stationMax}
        onChange={(e) =>
          setZoning({ stationMaxDistanceM: Number(e.target.value) })
        }
        aria-label="Station maximum distance"
        className="w-full"
      />
    </div>
  );
}

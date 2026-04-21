import { usePopulation } from "@/hooks/use-population";
import { usePrefectureStore } from "@/stores/prefecture-store";

interface PopulationSectionProps {
  areaCode: string;
}

/**
 * 人口・世帯数・空き家率セクション。3カラム表示。
 * Ref: DESIGN.md Sec 5.3 items 8-9
 */
export function PopulationSection({ areaCode }: PopulationSectionProps) {
  const prefCode = usePrefectureStore((s) => s.selectedPrefCode);
  const { data: populationData } = usePopulation(prefCode);

  const cityData = populationData?.find((p) => p.city_code === areaCode);

  return (
    <div className="p-4">
      <p
        className="text-xs font-medium tracking-wider mb-3"
        style={{ color: "var(--ts-text-muted)", letterSpacing: "0.5px" }}
      >
        人口・世帯
      </p>

      <div className="grid grid-cols-3 gap-3">
        <StatColumn
          label="人口"
          value={
            cityData?.population != null
              ? `${(cityData.population / 10000).toFixed(1)}万人`
              : "—"
          }
        />
        <StatColumn
          label="世帯数"
          value={
            cityData?.households != null
              ? `${(cityData.households / 10000).toFixed(1)}万`
              : "—"
          }
        />
        <StatColumn
          label="空き家率"
          value="—"
          // TODO: useVacancy hook を接続
        />
      </div>
    </div>
  );
}

function StatColumn({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <p className="text-[10px] mb-0.5" style={{ color: "var(--ts-text-dim)" }}>
        {label}
      </p>
      <p
        className="text-sm font-mono font-medium"
        style={{ color: "var(--ts-text-primary)" }}
      >
        {value}
      </p>
    </div>
  );
}

import { useMunicipalities } from "@/features/municipalities/api/use-municipalities";
import { useTransactionSummary } from "@/features/transactions/api/use-transaction-summary";
import { useMapStore } from "@/stores/map-store";
import { usePrefectureStore } from "@/stores/prefecture-store";
import { useUIStore } from "@/stores/ui-store";

export function MunicipalityList() {
  const prefCode = usePrefectureStore((s) => s.selectedPrefCode);
  const activeTheme = useUIStore((s) => s.activeTheme);
  const {
    data: municipalities,
    isLoading,
    isError,
  } = useMunicipalities(prefCode);
  const { data: transactionSummary } = useTransactionSummary(
    activeTheme === "transactions" ? prefCode : null,
  );
  const selectArea = useMapStore((s) => s.selectArea);
  const setViewState = useMapStore((s) => s.setViewState);
  const viewState = useMapStore((s) => s.viewState);

  if (isLoading) {
    return (
      <div className="p-4 text-sm text-white/60" aria-busy="true">
        読み込み中...
      </div>
    );
  }

  if (isError) {
    return (
      <div className="p-4 text-sm text-red-400/80" role="alert">
        データの取得に失敗しました
      </div>
    );
  }

  if (!municipalities?.length) {
    return <div className="p-4 text-sm text-white/60">データなし</div>;
  }

  const handleSelect = (m: (typeof municipalities)[number]) => {
    selectArea({
      code: m.city_code,
      name: m.city_name,
      level: "municipality",
      // bbox は別途 WASM/API から取得予定。暫定値として現在の viewport bbox を使用
      bbox: {
        south: viewState.latitude - 0.05,
        west: viewState.longitude - 0.05,
        north: viewState.latitude + 0.05,
        east: viewState.longitude + 0.05,
      },
    });
    setViewState({
      ...viewState,
      zoom: 12,
    });
  };

  return (
    <ul
      className="flex flex-col gap-1 p-2 overflow-y-auto max-h-[calc(100vh-200px)]"
      aria-label="市区町村一覧"
    >
      {municipalities.map((m) => {
        // "transactions" テーマ時の取引件数表示
        const txCount =
          activeTheme === "transactions"
            ? (transactionSummary?.find((t) => t.city_code === m.city_code)
                ?.tx_count ?? null)
            : null;

        return (
          <li key={m.city_code}>
            <button
              type="button"
              onClick={() => handleSelect(m)}
              className="flex w-full items-center justify-between px-3 py-2 rounded-lg hover:bg-white/10 text-sm text-left transition-colors"
            >
              <span className="text-white/80">{m.city_name}</span>
              {/* TODO: DataMode に応じたメトリクス表示 */}
              {txCount !== null && (
                <span className="text-white/50 text-xs tabular-nums">
                  {txCount.toLocaleString("ja-JP")}件
                </span>
              )}
            </button>
          </li>
        );
      })}
    </ul>
  );
}

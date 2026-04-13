"use client";

import { usePrefectureStore } from "@/stores/prefecture-store";
import { useMapStore } from "@/stores/map-store";
import { useDataModeStore } from "@/stores/data-mode-store";
import { useMunicipalities } from "@/features/municipalities/api/use-municipalities";
import { useTransactionSummary } from "@/features/transactions/api/use-transaction-summary";

export function MunicipalityList() {
  const prefCode = usePrefectureStore((s) => s.selectedPrefCode);
  const { data: municipalities, isLoading, isError } = useMunicipalities(prefCode);
  const { data: transactionSummary } = useTransactionSummary(
    dataMode === "transactions" ? prefCode : null,
  );
  const selectArea = useMapStore((s) => s.selectArea);
  const setViewState = useMapStore((s) => s.setViewState);
  const viewState = useMapStore((s) => s.viewState);
  const dataMode = useDataModeStore((s) => s.mode);

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
    return (
      <div className="p-4 text-sm text-white/60">
        データなし
      </div>
    );
  }

  return (
    <div
      className="flex flex-col gap-1 p-2 overflow-y-auto max-h-[calc(100vh-200px)]"
      role="list"
      aria-label="市区町村一覧"
    >
      {municipalities.map((m) => {
        // DataMode "transactions" 時の取引件数表示
        const txCount =
          dataMode === "transactions"
            ? (transactionSummary?.find((t) => t.city_code === m.city_code)
                ?.tx_count ?? null)
            : null;

        return (
          <button
            key={m.city_code}
            type="button"
            role="listitem"
            onClick={() => {
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
            }}
            className="flex items-center justify-between px-3 py-2 rounded-lg hover:bg-white/10 text-sm text-left transition-colors"
          >
            <span className="text-white/80">{m.city_name}</span>
            {/* TODO: DataMode に応じたメトリクス表示 */}
            {txCount !== null && (
              <span className="text-white/50 text-xs tabular-nums">
                {txCount.toLocaleString("ja-JP")}件
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}

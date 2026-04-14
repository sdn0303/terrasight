import { BottomSheet } from "@/components/layout/bottom-sheet";
import { DenseTable } from "@/components/table/dense-table";
import { propertyColumns } from "@/components/table/property-columns";
import { useOpportunities } from "@/hooks/use-opportunities";
import { GLOW_SHADOW, GRADIENT } from "@/lib/theme-tokens";
import { useFilterStore } from "@/stores/filter-store";
import { useUIStore } from "@/stores/ui-store";

interface OpportunitiesSheetProps {
  open: boolean;
  onClose: () => void;
}

/**
 * Composition of BottomSheet + DenseTable fed by {@link useOpportunities}.
 * Row click opens the Insight drawer for the clicked property by setting
 * `insight = { kind: "property", id, lat, lng }`.
 */
export function OpportunitiesSheet({ open, onClose }: OpportunitiesSheetProps) {
  const heightPct = useUIStore((s) => s.bottomSheetHeightPct);
  const setHeightPct = useUIStore((s) => s.setBottomSheetHeightPct);
  const insight = useUIStore((s) => s.insight);
  const setInsight = useUIStore((s) => s.setInsight);
  const activeFilterCount = useFilterStore((s) => s.activeCount());

  const query = useOpportunities(open);
  const data = query.data?.items ?? [];
  const total = query.data?.total ?? 0;

  const selectedId =
    insight !== null && insight.kind === "property" ? insight.id : null;

  if (!open) return null;

  return (
    <BottomSheet
      open={open}
      onClose={onClose}
      title="Opportunities"
      subtitle={
        query.isLoading
          ? "Loading…"
          : total > 0
            ? `${total} properties · Current viewport`
            : "No properties match current filters"
      }
      heightPct={heightPct}
      onHeightChange={setHeightPct}
      actions={
        <>
          <button
            type="button"
            className="rounded-[10px] px-3 py-2 text-[10px]"
            style={{
              background: "var(--neutral-50)",
              color: "var(--neutral-600)",
            }}
          >
            ⚙ Filter
            {activeFilterCount > 0 && (
              <span
                className="ml-1 inline-block rounded-full px-1.5 text-[8px] font-extrabold text-white"
                style={{ background: "var(--brand-indigo)" }}
              >
                {activeFilterCount}
              </span>
            )}
          </button>
          <button
            type="button"
            className="rounded-[10px] px-3 py-2 text-[10px] font-extrabold text-white"
            style={{
              background: GRADIENT.brand,
              boxShadow: GLOW_SHADOW.primarySubtle,
            }}
          >
            ↓ Export
          </button>
        </>
      }
    >
      {query.isLoading ? (
        <LoadingSkeleton />
      ) : query.error !== null ? (
        <ErrorState onRetry={() => query.refetch()} />
      ) : data.length === 0 ? (
        <EmptyState />
      ) : (
        <DenseTable
          columns={propertyColumns}
          data={data}
          selectedId={selectedId}
          getRowId={(row) => String(row.id)}
          onRowClick={(row) =>
            setInsight({
              kind: "property",
              id: String(row.id),
              lat: row.lat,
              lng: row.lng,
            })
          }
        />
      )}
    </BottomSheet>
  );
}

function LoadingSkeleton() {
  return (
    <div
      className="space-y-2 p-4"
      role="status"
      aria-label="Loading opportunities"
    >
      {Array.from({ length: 8 }).map((_, i) => (
        <div
          // biome-ignore lint/suspicious/noArrayIndexKey: static placeholder rows
          key={i}
          className="h-10 animate-pulse rounded"
          style={{ background: "var(--neutral-100)" }}
        />
      ))}
    </div>
  );
}

function EmptyState() {
  return (
    <div
      className="flex h-full items-center justify-center p-8 text-center text-[11px]"
      style={{ color: "var(--neutral-500)" }}
    >
      条件に一致する物件がありません。フィルターを調整してください。
    </div>
  );
}

function ErrorState({ onRetry }: { onRetry: () => void }) {
  return (
    <div
      className="flex h-full flex-col items-center justify-center gap-3 p-8 text-center text-[11px]"
      style={{ color: "var(--neutral-500)" }}
    >
      <div>読み込みに失敗しました</div>
      <button
        type="button"
        onClick={onRetry}
        className="rounded-[10px] px-3 py-2 text-[10px] font-bold"
        style={{
          background: "var(--neutral-100)",
          color: "var(--neutral-700)",
        }}
      >
        再試行
      </button>
    </div>
  );
}

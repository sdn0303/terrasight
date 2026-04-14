import { useQuery } from "@tanstack/react-query";
import { z } from "zod";
import { TransactionSummarySchema } from "@/lib/api/schemas/transaction";
import { queryKeys } from "@/lib/query-keys";

const API_BASE = import.meta.env.VITE_API_URL ?? "http://localhost:8000";

async function fetchTransactionSummary(
  prefCode: string,
  signal?: AbortSignal,
): Promise<z.infer<typeof TransactionSummarySchema>[]> {
  const url = `${API_BASE}/api/v1/transactions/summary?pref_code=${encodeURIComponent(prefCode)}`;
  const res = await fetch(url, { signal: signal ?? null });
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  const raw: unknown = await res.json();
  return z.array(TransactionSummarySchema).parse(raw);
}

export function useTransactionSummary(prefCode: string | null) {
  return useQuery({
    queryKey: prefCode
      ? queryKeys.transactionSummary.byPref(prefCode)
      : queryKeys.transactionSummary.all,
    queryFn: ({ signal }) => fetchTransactionSummary(prefCode!, signal),
    enabled: prefCode !== null,
    staleTime: 60_000,
  });
}

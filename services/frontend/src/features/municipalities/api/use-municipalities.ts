import { useQuery } from "@tanstack/react-query";
import { z } from "zod";
import { MunicipalitySchema } from "@/lib/api/schemas/municipality";
import { queryKeys } from "@/lib/query-keys";

const API_BASE = import.meta.env.VITE_API_URL ?? "http://localhost:8000";

async function fetchMunicipalities(
  prefCode: string,
  signal?: AbortSignal,
): Promise<z.infer<typeof MunicipalitySchema>[]> {
  const url = `${API_BASE}/api/v1/municipalities?pref_code=${encodeURIComponent(prefCode)}`;
  const res = await fetch(url, { signal: signal ?? null });
  if (!res.ok) throw new Error(`API error: ${res.status}`);
  const raw: unknown = await res.json();
  return z.array(MunicipalitySchema).parse(raw);
}

export function useMunicipalities(prefCode: string | null) {
  return useQuery({
    queryKey: prefCode
      ? queryKeys.municipalities.byPref(prefCode)
      : queryKeys.municipalities.all,
    queryFn: ({ signal }) => {
      if (prefCode === null) throw new Error("prefCode is required");
      return fetchMunicipalities(prefCode, signal);
    },
    enabled: prefCode !== null,
    staleTime: 300_000, // 市区町村リストは頻繁に変わらない
  });
}

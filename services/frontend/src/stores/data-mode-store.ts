import { create } from "zustand";
import { devtools } from "zustand/middleware";

export type DataMode =
  | "tls"
  | "land-price"
  | "yield"
  | "risk"
  | "population"
  | "transactions"
  | "hazard";

export interface DataModeConfig {
  id: DataMode;
  label: string;
  labelJa: string;
  available: boolean;
}

export const DATA_MODES: readonly DataModeConfig[] = [
  { id: "tls", label: "TLS Score", labelJa: "TLS総合スコア", available: true },
  {
    id: "land-price",
    label: "Land Price",
    labelJa: "基準地価",
    available: true,
  },
  { id: "yield", label: "Yield", labelJa: "利回り", available: false },
  { id: "risk", label: "Risk", labelJa: "リスク", available: true },
  { id: "population", label: "Population", labelJa: "人口", available: false },
  {
    id: "transactions",
    label: "Transactions",
    labelJa: "取引事例",
    available: true,
  },
  {
    id: "hazard",
    label: "Hazard Map",
    labelJa: "ハザードマップ",
    available: true,
  },
] as const;

interface DataModeState {
  mode: DataMode;
  setMode: (mode: DataMode) => void;
}

export const useDataModeStore = create<DataModeState>()(
  devtools(
    (set) => ({
      mode: "tls",
      setMode: (mode) => set({ mode }),
    }),
    { name: "data-mode-store" },
  ),
);

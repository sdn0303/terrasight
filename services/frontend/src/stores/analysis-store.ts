import { create } from "zustand";
import { devtools } from "zustand/middleware";

export type WeightPreset = "balance" | "investment" | "residential" | "disaster";

export interface AnalysisPoint {
  lat: number;
  lng: number;
  address?: string;
}

interface AnalysisState {
  weightPreset: WeightPreset;
  setWeightPreset: (preset: WeightPreset) => void;
  analysisPoint: AnalysisPoint | null;
  setAnalysisPoint: (point: AnalysisPoint | null) => void;
  analysisRadius: number;
  setAnalysisRadius: (radius: number) => void;
}

export const useAnalysisStore = create<AnalysisState>()(
  devtools(
    (set) => ({
      weightPreset: "balance",
      setWeightPreset: (preset) => set({ weightPreset: preset }),
      analysisPoint: null,
      setAnalysisPoint: (point) => set({ analysisPoint: point }),
      analysisRadius: 500,
      setAnalysisRadius: (radius) => set({ analysisRadius: radius }),
    }),
    { name: "analysis-store" },
  ),
);

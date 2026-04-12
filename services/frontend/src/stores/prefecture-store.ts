import { create } from "zustand";
import { devtools } from "zustand/middleware";

interface PrefectureState {
  /** Selected prefecture code (2-digit, e.g. "13") */
  selectedPrefCode: string;
  /** Selected prefecture display name (e.g. "東京都") */
  selectedPrefName: string;
  /** Set selected prefecture — triggers map fly-to + layer reload in consuming components */
  selectPrefecture: (code: string, name: string) => void;
}

export const usePrefectureStore = create<PrefectureState>()(
  devtools(
    (set) => ({
      selectedPrefCode: "13",
      selectedPrefName: "東京都",
      selectPrefecture: (code, name) =>
        set({ selectedPrefCode: code, selectedPrefName: name }),
    }),
    { name: "prefecture-store" },
  ),
);

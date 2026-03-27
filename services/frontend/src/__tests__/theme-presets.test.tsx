import { describe, expect, it, vi } from "vitest";
import { getLayerIdsByTheme } from "@/lib/themes";

// Mock next-intl
vi.mock("next-intl", () => ({
  useTranslations: () => (key: string) => {
    const map: Record<string, string> = {
      "theme.safety": "安全性",
      "theme.livability": "利便性",
      "theme.price": "価格",
      "theme.future": "将来性",
      "theme.safety.desc": "洪水・地震・急傾斜地などの災害リスクを可視化",
      "theme.livability.desc": "学校・医療・鉄道などの生活インフラを表示",
      "theme.price.desc": "地価公示・用途地域の投資価値データ",
      "theme.future.desc": "人口推移・都市計画の将来性指標",
    };
    return map[key] ?? key;
  },
}));

describe("Theme layer counts", () => {
  it("safety theme has layers", () => {
    const layers = getLayerIdsByTheme("safety");
    expect(layers.length).toBeGreaterThan(0);
  });

  it("all four themes have at least one layer", () => {
    for (const id of ["safety", "livability", "price", "future"] as const) {
      expect(getLayerIdsByTheme(id).length).toBeGreaterThan(0);
    }
  });
});

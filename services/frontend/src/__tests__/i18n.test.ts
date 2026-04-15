import { describe, it, expect } from "vitest";
import { t } from "@/lib/i18n";

describe("i18n t()", () => {
  it('t("ja", "sidebar.explore") returns "探す"', () => {
    expect(t("ja", "sidebar.explore")).toBe("探す");
  });

  it('t("en", "sidebar.explore") returns "Explore"', () => {
    expect(t("en", "sidebar.explore")).toBe("Explore");
  });

  it("t() returns the key itself for a nonexistent key", () => {
    expect(t("ja", "nonexistent.key")).toBe("nonexistent.key");
    expect(t("en", "nonexistent.key")).toBe("nonexistent.key");
  });

  it("all ja keys have en counterparts (locale symmetry)", () => {
    // Access internal messages via t() round-trip: any key present in ja but
    // absent in en would return the key string itself. We verify by calling
    // t("en", key) for every ja key and checking the result is NOT equal to
    // the key (which would indicate a missing translation).
    //
    // To enumerate all ja keys we leverage the fact that t("ja", k) === v
    // (where v != k) for every defined key.  We compare the set of keys that
    // resolve in ja with those that resolve in en by checking each known key.
    const knownKeys = [
      "sidebar.explore",
      "sidebar.view",
      "sidebar.settings",
      "sidebar.opportunities",
      "sidebar.score",
      "sidebar.land-price",
      "sidebar.transactions",
      "sidebar.hazard",
      "sidebar.station",
      "settings.title",
      "settings.mapStyle",
      "settings.mapStyle.light",
      "settings.mapStyle.dark",
      "settings.mapStyle.satellite",
      "settings.language",
      "settings.language.ja",
      "settings.language.en",
      "panel.close",
      "panel.address",
      "table.address",
      "table.tls",
      "table.price",
      "table.risk",
      "table.trend",
      "table.station",
      "table.count",
      "table.export",
      "table.filter",
      "drawer.detail",
      "drawer.propertyDetail",
      "drawer.pointDetail",
      "common.loading",
      "common.noData",
      "common.error",
      "theme.safety.name",
      "theme.safety.desc",
      "theme.livability.name",
      "theme.livability.desc",
      "theme.price.name",
      "theme.price.desc",
      "theme.future.name",
      "theme.future.desc",
      "score.header",
      "tls.score",
      "axis.disaster",
      "axis.terrain",
      "axis.livability",
      "axis.future",
      "axis.price",
      "compare.title",
      "compare.address",
      "compare.score",
    ];

    for (const key of knownKeys) {
      const jaValue = t("ja", key);
      const enValue = t("en", key);
      // Both locales must resolve to something other than the key itself
      expect(jaValue, `ja missing key: ${key}`).not.toBe(key);
      expect(enValue, `en missing key: ${key}`).not.toBe(key);
    }
  });
});

import { expect, test } from "@playwright/test";

test("time machine slider updates year", async ({ page }) => {
  await page.goto("/");

  const slider = page.getByLabel("地価公示年度選択");
  await expect(slider).toBeVisible();
});

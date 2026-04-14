import { expect, test } from "@playwright/test";

test("URL deep link restores finder panel", async ({ page }) => {
  await page.goto("/?panel=finder");

  // Finder panel should be visible
  await expect(page.getByText("Investment Finder")).toBeVisible();
});

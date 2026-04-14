import { expect, test } from "@playwright/test";

test.describe("Full user flow", () => {
  test("Finder → filter → Opportunities → drawer", async ({ page }) => {
    await page.goto("/");

    // Map loads
    await expect(
      page.locator('[aria-label="Terrasight map canvas"]'),
    ).toBeVisible();

    // Rail is visible
    await expect(page.getByRole("button", { name: "Finder" })).toBeVisible();

    // Open Finder
    await page.getByRole("button", { name: "Finder" }).click();
    await expect(page.getByText("Investment Finder")).toBeVisible();

    // Search (opens opportunities sheet)
    await page.getByRole("button", { name: /物件を検索/ }).click();
    await expect(page.getByText("Opportunities")).toBeVisible();

    // Click a row to open drawer
    const firstRow = page.locator("tbody tr").first();
    await firstRow.click();

    // Drawer opens (aside element = complementary role)
    await expect(page.getByRole("complementary")).toBeVisible();

    // Close drawer
    await page.getByRole("button", { name: /Close drawer/ }).click();
    await expect(page.getByRole("complementary")).not.toBeVisible();
  });
});

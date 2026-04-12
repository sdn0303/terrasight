import { expect, test } from "@playwright/test";

test.describe("Compare flow", () => {
  test("multi-select 2 rows → Compare tab shows", async ({ page }) => {
    await page.goto("/");

    // Open Opportunities via rail button
    await page.getByRole("button", { name: "Opportunities" }).click();
    await expect(page.getByText("Opportunities")).toBeVisible();

    // First click a row to open the drawer
    const rows = page.locator("tbody tr");
    await rows.first().click();
    await expect(page.getByRole("complementary")).toBeVisible();

    // Select 2 rows via checkboxes
    await rows.nth(0).getByRole("checkbox").click();
    await rows.nth(1).getByRole("checkbox").click();

    // Compare tab appears
    await expect(page.getByRole("tab", { name: "Compare" })).toBeVisible();
  });
});

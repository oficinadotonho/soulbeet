const { test, expect } = require("@playwright/test");

test("login form renders", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByText("Soulbeet")).toBeVisible();
  await expect(page.getByPlaceholder("Enter username")).toBeVisible();
  await expect(page.getByPlaceholder("Enter password")).toBeVisible();
  await expect(page.getByRole("button", { name: "AUTHENTICATE" })).toBeVisible();
});

test("history route requires auth", async ({ page }) => {
  await page.goto("/history");
  await expect(page.getByRole("button", { name: "AUTHENTICATE" })).toBeVisible();
});

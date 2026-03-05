const { test, expect } = require("@playwright/test");

test("root route serves app shell", async ({ page }) => {
  const response = await page.goto("/");
  expect(response?.ok()).toBeTruthy();
  await expect(page.getByRole("heading", { name: "SoulBeet" })).toBeVisible();
});

test("history route serves app shell", async ({ page }) => {
  const response = await page.goto("/history");
  expect(response?.ok()).toBeTruthy();
  await expect(page.getByRole("heading", { name: "SoulBeet" })).toBeVisible();
});

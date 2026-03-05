import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "tests/e2e",
  timeout: 30000,
  retries: 0,
  use: {
    baseURL: process.env.E2E_BASE_URL || "http://127.0.0.1:9765",
    headless: true,
  },
  webServer: {
    command:
      "mkdir -p target/debug/public && PORT=9765 IP=127.0.0.1 cargo run -p web --features server",
    url: process.env.E2E_BASE_URL || "http://127.0.0.1:9765",
    reuseExistingServer: true,
    timeout: 240000,
  },
  reporter: [["list"]],
});

import { defineConfig } from "@playwright/test";

// Browser tests drive the real app against a throw-away server (see e2e/serve.sh).
export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: [["list"]],
  use: { baseURL: "http://127.0.0.1:3123", trace: "retain-on-failure", screenshot: "only-on-failure" },
  webServer: { command: "bash e2e/serve.sh", url: "http://127.0.0.1:3123/healthz", timeout: 240_000, reuseExistingServer: !process.env.CI },
  outputDir: "e2e/.tmp/results",
});

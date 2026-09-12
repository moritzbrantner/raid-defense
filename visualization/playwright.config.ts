import { defineConfig } from "@playwright/test";

const port = Number(process.env.PLAYWRIGHT_PORT ?? 5173);

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL ?? `http://127.0.0.1:${port}`,
    headless: true,
  },
  webServer: {
    command: `bun run dev -- --port ${port}`,
    port,
    reuseExistingServer: false,
    timeout: 120_000,
  },
});

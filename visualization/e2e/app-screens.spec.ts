import { expect, type Page, test } from "@playwright/test";
import fs from "node:fs/promises";

async function resetApp(page: Page) {
  await page.goto("/");
  await page.evaluate(() => window.localStorage.clear());
  await page.reload();
  await expect(page.getByRole("heading", { name: "Fresh Frontier" })).toBeVisible();
}

async function openSimulationLab(page: Page) {
  await page.getByRole("button", { name: "Open Lab" }).click();
  await expect(page.getByText("Simulation Lab").first()).toBeVisible();
  await expect(page.locator('[data-testid="simulation-clock"]')).toHaveText("0:00");
}

async function openMainCampaign(page: Page) {
  await page.getByRole("button", { name: "Start Campaign" }).click();
  await expect(page.getByText("Main Campaign").first()).toBeVisible();
  await expect(page.locator('[data-testid="simulation-clock"]')).toHaveText("0:00");
}

test.beforeEach(async ({ page }) => {
  await resetApp(page);
});

test("home screen exposes every top-level destination", async ({ page }) => {
  await expect(page.getByRole("heading", { name: "Fresh Frontier" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Simulation Lab" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Control Room" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Field Manual" })).toBeVisible();

  await page.getByRole("button", { name: "Open Settings" }).click();
  await expect(page.getByRole("heading", { name: "Control Room" })).toBeVisible();
  await page.getByRole("button", { name: "Back Home" }).click();
  await expect(page.getByRole("button", { name: "Start Campaign" })).toBeVisible();

  await openMainCampaign(page);
  await page.getByRole("button", { name: "Menu" }).click();
  await expect(page.locator('[data-testid="pause-menu"]')).toBeVisible();
  await page.getByRole("button", { name: "Home", exact: true }).click();
  await expect(page.getByRole("button", { name: "Continue Campaign" })).toBeVisible();

  await page.getByRole("button", { name: "Open Lab" }).click();
  await expect(page.getByText("Simulation Lab").first()).toBeVisible();

  await page.getByRole("button", { name: "Menu" }).click();
  await page.getByRole("button", { name: "Wiki" }).click();
  await expect(page.getByRole("heading", { name: "Field Manual" })).toBeVisible();
});

test("running game keeps the main menu compact and pauses from menu or escape", async ({ page }) => {
  await openMainCampaign(page);

  await expect(page.getByRole("button", { name: "Menu" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Home", exact: true })).toBeHidden();

  await page.keyboard.press("Escape");
  await expect(page.locator('[data-testid="pause-menu"]')).toBeVisible();
  await expect(page.getByText("Paused")).toBeVisible();

  await page.getByRole("button", { name: "Resume Game" }).click();
  await expect(page.locator('[data-testid="pause-menu"]')).toBeHidden();

  await page.getByRole("button", { name: "Menu" }).click();
  await expect(page.locator('[data-testid="pause-menu"]')).toBeVisible();
});

test("settings screen persists interface choices and seeds", async ({ page }) => {
  await page.getByRole("button", { name: "Open Settings" }).click();

  await page.getByLabel("Default Main Game Seed").fill("777");
  await page.getByLabel("Default Simulation Seed").fill("888");
  await page.getByRole("button", { name: /Ambient Overlay/ }).click();
  await page.getByRole("button", { name: /Status Notifications/ }).click();

  await page.reload();
  await page.getByRole("button", { name: "Open Settings" }).click();

  await expect(page.getByLabel("Default Main Game Seed")).toHaveValue("777");
  await expect(page.getByLabel("Default Simulation Seed")).toHaveValue("888");
  await expect(page.getByRole("button", { name: /Ambient Overlay/ })).toContainText("Disabled");
  await expect(page.getByRole("button", { name: /Status Notifications/ })).toContainText(
    "Disabled",
  );
});

test("settings screen falls back safely when a default seed is invalid", async ({ page }) => {
  await page.getByRole("button", { name: "Open Settings" }).click();
  await page.getByLabel("Default Simulation Seed").fill("not-a-seed");
  await page.getByRole("button", { name: "Back Home" }).click();

  await openSimulationLab(page);

  await expect(page.getByPlaceholder("Seed")).toHaveValue("2106659303718057601");
  await expect(page.locator('[data-testid="status-banner"]')).toContainText(
    "Simulation lab opened.",
  );
});

test("wiki screen shows an empty manual before a run and logs encountered units during a run", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Open Wiki" }).click();
  await expect(page.getByRole("heading", { name: "Field Manual" })).toBeVisible();
  await expect(page.getByText("No Active Frontier")).toBeVisible();
  await expect(page.getByText("Open a campaign or simulation first.")).toBeVisible();

  await resetApp(page);
  await openSimulationLab(page);
  await page.getByRole("button", { name: "Recruit Worker" }).click();
  await expect(page.locator('[data-testid="status-banner"]')).toContainText(
    "Recruit Worker queued.",
  );

  await page.getByRole("button", { name: "Menu" }).click();
  await page.getByRole("button", { name: "Wiki" }).click();
  await expect(page.getByRole("heading", { name: "Field Manual" })).toBeVisible();
  await expect(page.getByText("Worker").first()).toBeVisible();
  await expect(page.getByText(/\d+ active/)).toBeVisible();
  await expect(
    page.getByText("Workers keep construction, hauling, and food consumption moving through the settlement core."),
  ).toBeVisible();
});

test("main campaign screen advances time and resumes the autosaved run", async ({ page }) => {
  await openMainCampaign(page);

  await page.getByRole("button", { name: "+30s" }).click();
  await expect(page.locator('[data-testid="simulation-clock"]')).toHaveText("0:30");
  await expect(page.locator('[data-testid="status-banner"]')).toContainText(
    "Campaign advanced by +30s.",
  );

  await page.getByRole("button", { name: "Menu" }).click();
  await page.getByRole("button", { name: "Home", exact: true }).click();
  await expect(page.getByRole("button", { name: "Continue Campaign" })).toBeVisible();

  await page.getByRole("button", { name: "Continue Campaign" }).click();
  await expect(page.getByText("Main Campaign").first()).toBeVisible();
  await expect(page.locator('[data-testid="simulation-clock"]')).toHaveText("0:30");
});

test("simulation screen saves, advances, and reloads a snapshot", async ({ page }) => {
  await openSimulationLab(page);

  await page.getByRole("button", { name: "+100 Wood" }).click();
  await expect(page.locator('[data-testid="status-banner"]')).toContainText("+100 Wood applied.");

  await page.locator('[data-testid="simulation-save-name"]').fill("Opening Save");
  await page.locator('[data-testid="simulation-save-current"]').click();
  await expect(page.locator('[data-testid="status-banner"]')).toContainText("Saved Opening Save.");
  await expect(page.getByText("Opening Save", { exact: true })).toBeVisible();

  await page.getByRole("button", { name: "+30s" }).click();
  await expect(page.locator('[data-testid="simulation-clock"]')).toHaveText("0:30");

  await page.getByRole("button", { name: "Load Save State" }).click();
  await expect(page.locator('[data-testid="status-banner"]')).toContainText("Loaded Opening Save.");
  await expect(page.locator('[data-testid="simulation-clock"]')).toHaveText("0:00");
});

test("simulation screen reports invalid seeds and malformed imports", async ({ page }, testInfo) => {
  await openSimulationLab(page);

  await page.getByPlaceholder("Seed").fill("");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(page.locator('[data-testid="status-banner"]')).toContainText(
    "Enter a valid non-negative seed.",
  );

  await page.getByPlaceholder("Seed").fill("-1");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect(page.locator('[data-testid="status-banner"]')).toContainText(
    "Enter a valid non-negative seed.",
  );

  const invalidSavePath = testInfo.outputPath("invalid-simulation.json");
  await fs.writeFile(invalidSavePath, "{not-json", "utf8");

  await page.locator('[data-testid="simulation-import-input"]').setInputFiles(invalidSavePath);
  await expect(page.locator('[data-testid="status-banner"]')).toContainText(
    "Simulation file is not valid JSON.",
  );
});

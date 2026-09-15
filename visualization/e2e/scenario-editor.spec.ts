import { expect, test } from "@playwright/test";

async function openScenarioEditor(page: import("@playwright/test").Page) {
  await page.goto("/?screen=settings&section=scenario");
  await expect(page.getByTestId("settings-screen")).toBeVisible();
  await expect(page.getByTestId("scenario-editor")).toBeVisible();
}

test("wave editor exposes exact ordered raider composition", async ({ page }) => {
  await openScenarioEditor(page);

  await page.getByTestId("scenario-wave-5").click();

  await expect(page.getByTestId("scenario-wave-group-type-0")).toHaveValue("basic");
  await expect(page.getByTestId("scenario-wave-group-count-0")).toHaveValue("2");
  await expect(page.getByTestId("scenario-wave-group-type-1")).toHaveValue("advanced");
  await expect(page.getByTestId("scenario-wave-group-count-1")).toHaveValue("1");

  await page.getByTestId("scenario-wave-group-count-0").fill("3");
  await page.getByTestId("scenario-wave-group-count-1").fill("2");
  await expect(page.getByTestId("scenario-wave-5")).toContainText("5 raiders");
});

test("scenario editor exposes unit and tower rules instead of difficulty presets", async ({ page }) => {
  await openScenarioEditor(page);

  await page.getByTestId("scenario-units-tab").click();
  const advanced = page.getByTestId("scenario-unit-advanced");
  await expect(advanced.getByLabel("Health")).toHaveValue("60");
  await advanced.getByLabel("Health").fill("75");
  await expect(advanced.getByLabel("Health")).toHaveValue("75");

  await page.getByTestId("scenario-towers-tab").click();
  const towerEditor = page.getByTestId("scenario-tower-editor");
  await expect(towerEditor.getByLabel("Build cost")).toHaveValue("25");
  await towerEditor.getByLabel("Build cost").fill("30");
  await expect(towerEditor.getByLabel("Build cost")).toHaveValue("30");

  await expect(page.getByTestId("scenario-settings")).not.toContainText("Raid size");
  await expect(page.getByTestId("scenario-settings")).not.toContainText("Raider strength");
});

test("world scenario controls expose explicit values instead of presets", async ({ page }) => {
  await openScenarioEditor(page);

  await page.getByTestId("scenario-world-tab").click();
  const worldEditor = page.getByTestId("scenario-world-editor");
  await expect(worldEditor.locator("select")).toHaveCount(0);

  const startingWood = page.getByTestId("scenario-starting-wood");
  const regrowthInterval = page.getByTestId("scenario-forest-regrowth-interval");
  const dayLength = page.getByTestId("scenario-day-length-ticks");
  await expect(startingWood).toHaveValue("120");
  await expect(regrowthInterval).toHaveValue("20");
  await expect(dayLength).toHaveValue("600");

  await startingWood.fill("240");
  await regrowthInterval.fill("11");
  await dayLength.fill("875");
  await expect(startingWood).toHaveValue("240");
  await expect(regrowthInterval).toHaveValue("11");
  await expect(dayLength).toHaveValue("875");

  const automaticRaids = page.getByTestId("scenario-automatic-raids");
  await expect(automaticRaids).toBeChecked();
  await automaticRaids.uncheck();
  await expect(automaticRaids).not.toBeChecked();
});

import { expect, test } from "@playwright/test";
import fs from "node:fs/promises";
import path from "node:path";

const fixturesDir = new URL("./fixtures/", import.meta.url);

async function readFixture(name: string) {
  const fixturePath = path.join(fixturesDir.pathname, `${name}.json`);
  return {
    fixturePath,
    fixtureJson: JSON.parse(await fs.readFile(fixturePath, "utf8")),
  };
}

test("imports a saved simulation JSON through the UI", async ({ page }) => {
  const { fixturePath } = await readFixture("claimed-province");

  await page.goto("/");
  await page.getByRole("button", { name: "Open Lab" }).click();

  await page.locator('[data-testid="simulation-import-input"]').setInputFiles(fixturePath);

  await expect(page.locator('[data-testid="status-banner"]')).toContainText("Loaded Claimed Province.");
  await expect(page.locator('[data-testid="simulation-clock"]')).toHaveText("1:24");
  await expect(page.getByRole("button", { name: /Vesper March/ })).toBeVisible();
});

test("loads fixture states directly through the e2e bridge and can export them again", async ({
  page,
}) => {
  const { fixtureJson } = await readFixture("fresh-frontier");

  await page.goto("/");

  await page.evaluate(async (fixture) => {
    await window.__RAID_DEFENSE_E2E__?.loadSimulationFile(fixture);
  }, fixtureJson);

  await expect(page.locator('[data-testid="simulation-clock"]')).toHaveText("0:00");

  const exported = await page.evaluate(() => window.__RAID_DEFENSE_E2E__?.exportCurrentSimulation());

  expect(exported).not.toBeNull();
  expect(exported?.format).toBe("raid-defense-simulation");
  expect(exported?.mode).toBe("simulation");
  expect(exported?.seed).toBe("42");
  expect(typeof exported?.snapshot_json).toBe("string");
  expect(exported?.snapshot_json.length).toBeGreaterThan(0);
});

test("first raider wave can destroy the starting castle and end the run in defeat", async ({
  page,
}) => {
  const { fixtureJson } = await readFixture("fresh-frontier");

  await page.goto("/");

  await page.evaluate(async (fixture) => {
    await window.__RAID_DEFENSE_E2E__?.loadSimulationFile(fixture);
  }, fixtureJson);

  const spawnResponse = await page.evaluate(() =>
    window.__RAID_DEFENSE_E2E__?.applyCommand({
      SpawnEntity: {
        blueprint: { Unit: "basic_raider" },
        name: null,
        location: { x: 0, y: 15, elevation: 0 },
      },
    }),
  );

  expect(spawnResponse?.accepted).toBe(true);
  expect(spawnResponse?.view.encountered_attack_waves).toHaveLength(1);

  const advanceResponse = await page.evaluate(() => window.__RAID_DEFENSE_E2E__?.advanceSimulation(20));

  expect(advanceResponse?.accepted).toBe(true);
  expect(advanceResponse?.view.summary.lost).toBe(true);
  expect(advanceResponse?.view.buildings.some((building) => building.kind === "castle")).toBe(false);

  await expect(page.getByText("The castle has fallen. The frontier is lost.")).toBeVisible();
});

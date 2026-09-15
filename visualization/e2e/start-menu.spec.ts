import { expect, test } from "@playwright/test";

test("starts a new game and resumes saved authoritative progress", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByTestId("start-menu")).toBeVisible();
  await expect(page.getByTestId("resume-game")).toBeDisabled();

  await page.getByTestId("new-game").click();
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();

  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("cycle-timer")).toContainText("Rally ·");
  await expect(page.getByTestId("wave-value")).toHaveText("0");
  await expect(page.getByTestId("cycle-timer")).toHaveText("Night · Wave 1", {
    timeout: 12_000,
  });
  await expect(page.getByTestId("wave-value")).toHaveText("1");

  await page.getByTestId("return-to-menu").click();
  await expect(page.getByTestId("start-menu")).toBeVisible();
  await expect(page.getByTestId("resume-game")).toBeEnabled();

  await page.reload();
  await expect(page.getByTestId("start-menu")).toBeVisible();
  await expect(page.getByTestId("resume-game")).toBeEnabled();

  await page.getByTestId("resume-game").click();
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("wave-value")).toHaveText("1");
  await expect(page.getByTestId("cycle-timer")).toHaveText("Night · Wave 1");
});

test("opens URL-addressable settings and persists presentation preferences", async ({ page }) => {
  await page.goto("/");

  await page.getByTestId("open-settings").click();
  await expect(page.getByTestId("settings-screen")).toBeVisible();
  await expect(page).toHaveURL(/screen=settings/);

  const touchHints = page.getByTestId("setting-touch-hints");
  const reduceMotion = page.getByTestId("setting-reduce-motion");
  const compactStatus = page.getByTestId("setting-compact-status");
  await expect(touchHints).toBeChecked();
  await expect(reduceMotion).not.toBeChecked();
  await expect(compactStatus).not.toBeChecked();

  await touchHints.uncheck();
  await reduceMotion.check();
  await compactStatus.check();
  await page.reload();

  await expect(page.getByTestId("settings-screen")).toBeVisible();
  await expect(touchHints).not.toBeChecked();
  await expect(reduceMotion).toBeChecked();
  await expect(compactStatus).toBeChecked();
});

test("starts and resumes a game with authoritative scenario options", async ({ page }) => {
  await page.goto("/?screen=settings&section=scenario");

  await expect(page.getByTestId("settings-screen")).toBeVisible();
  await expect(page.getByTestId("scenario-settings")).toBeVisible();
  await expect(page).toHaveURL(/section=scenario/);

  await page.getByTestId("scenario-wave-5").click();
  await page.getByTestId("scenario-wave-group-count-0").fill("3");
  await page.getByTestId("scenario-wave-group-count-1").fill("2");

  await page.getByTestId("scenario-units-tab").click();
  await page.getByTestId("scenario-unit-advanced").getByLabel("Health").fill("75");

  await page.getByTestId("scenario-towers-tab").click();
  await page.getByTestId("scenario-tower-editor").getByLabel("Build cost").fill("30");

  await page.getByTestId("scenario-world-tab").click();
  await page.getByTestId("scenario-starting-supplies").selectOption("rich");
  await page.getByTestId("scenario-forest-density").selectOption("dense");
  await page.getByTestId("scenario-forest-regrowth").selectOption("fast");
  await page.getByTestId("scenario-sawmill-throughput").selectOption("fast");
  await page.getByTestId("scenario-day-length").selectOption("long");
  await page.getByTestId("scenario-raid-timing").selectOption("manual");
  await page.getByTestId("scenario-raid-economy").selectOption("continuous");

  await page.getByRole("button", { name: "Back" }).click();
  await expect(page.getByTestId("start-menu")).toBeVisible();
  await page.getByTestId("new-game").click();
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("wood-value")).toHaveText("240");

  const savedScenario = await page.evaluate(() => {
    const raw = window.localStorage.getItem("raid-defense.save.v2");
    if (!raw) return null;
    const scenario = (JSON.parse(raw) as { scenario?: Record<string, any> }).scenario;
    if (!scenario) return null;
    return {
      starting_supplies: scenario.starting_supplies,
      forest_density: scenario.forest_density,
      forest_regrowth: scenario.forest_regrowth,
      sawmill_throughput: scenario.sawmill_throughput,
      raid_size: scenario.raid_size,
      raider_strength: scenario.raider_strength,
      day_length: scenario.day_length,
      raid_timing: scenario.raid_timing,
      raid_economy: scenario.raid_economy,
      wave5: scenario.waves?.[4],
      advanced_health: scenario.raiders?.advanced?.health,
      arrow_build_cost: scenario.towers?.arrow?.build_cost,
    };
  });
  expect(savedScenario).toEqual({
    starting_supplies: "rich",
    forest_density: "dense",
    forest_regrowth: "fast",
    sawmill_throughput: "fast",
    raid_size: "standard",
    raider_strength: "standard",
    day_length: "long",
    raid_timing: "manual",
    raid_economy: "continuous",
    wave5: {
      groups: [
        { archetype: "basic", count: 3 },
        { archetype: "advanced", count: 2 },
      ],
    },
    advanced_health: 75,
    arrow_build_cost: 30,
  });

  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("cycle-timer")).toContainText("Rally ·");
  await expect(page.getByTestId("wave-value")).toHaveText("0");
  await expect(page.getByTestId("cycle-timer")).toHaveText("Night · Wave 1", {
    timeout: 12_000,
  });
  await expect(page.getByTestId("wave-value")).toHaveText("1");
  await page.getByTestId("return-to-menu").click();
  await expect(page.getByTestId("resume-game")).toBeEnabled();

  await page.reload();
  await page.getByTestId("resume-game").click();
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("wood-value")).toHaveText("240");
  await expect(page.getByTestId("wave-value")).toHaveText("1");
  await expect(page.getByTestId("cycle-timer")).toHaveText("Night · Wave 1");
});

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

  await page.getByTestId("scenario-starting-supplies").selectOption("rich");
  await page.getByTestId("scenario-forest-density").selectOption("dense");
  await page.getByTestId("scenario-forest-regrowth").selectOption("fast");
  await page.getByTestId("scenario-sawmill-throughput").selectOption("fast");
  await page.getByTestId("scenario-raid-size").selectOption("large");
  await page.getByTestId("scenario-raider-strength").selectOption("harsh");
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
    return (JSON.parse(raw) as { version?: number; scenario?: unknown }).scenario;
  });
  expect(savedScenario).toEqual({
    starting_supplies: "rich",
    forest_density: "dense",
    forest_regrowth: "fast",
    sawmill_throughput: "fast",
    raid_size: "large",
    raider_strength: "harsh",
    day_length: "long",
    raid_timing: "manual",
    raid_economy: "continuous",
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

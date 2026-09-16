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
  await expect(page.getByTestId("shared-settings-fields").locator('[data-slot="toggle-setting"]')).toHaveCount(3);

  const touchHints = page.getByTestId("setting-touch-hints");
  const reduceMotion = page.getByTestId("setting-reduce-motion");
  const compactStatus = page.getByTestId("setting-compact-status");
  await expect(touchHints).toHaveAttribute("aria-checked", "true");
  await expect(reduceMotion).toHaveAttribute("aria-checked", "false");
  await expect(compactStatus).toHaveAttribute("aria-checked", "false");

  await touchHints.click();
  await reduceMotion.click();
  await compactStatus.click();

  await expect
    .poll(
      () =>
        page.evaluate(() => {
          const raw = window.localStorage.getItem("raid-defense.settings.user.v2");
          if (!raw) return null;
          const snapshot = JSON.parse(raw) as {
            schema_version?: number;
            scope?: string;
            overrides?: Record<string, { type?: string; value?: unknown }>;
          };
          return {
            schemaVersion: snapshot.schema_version,
            scope: snapshot.scope,
            touchHints: snapshot.overrides?.["presentation.show_touch_hints"]?.value,
            reduceMotion: snapshot.overrides?.["accessibility.reduce_motion"]?.value,
            compactStatus: snapshot.overrides?.["presentation.compact_status"]?.value,
            hasScenarioSettings: Object.keys(snapshot.overrides ?? {}).some((key) =>
              key.startsWith("scenario."),
            ),
          };
        }),
      { timeout: 10_000 },
    )
    .toEqual({
      schemaVersion: 2,
      scope: "user",
      touchHints: false,
      reduceMotion: true,
      compactStatus: true,
      hasScenarioSettings: false,
    });

  await page.reload();

  await expect(page.getByTestId("settings-screen")).toBeVisible();
  await expect(touchHints).toHaveAttribute("aria-checked", "false");
  await expect(reduceMotion).toHaveAttribute("aria-checked", "true");
  await expect(compactStatus).toHaveAttribute("aria-checked", "true");
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
  await page.getByTestId("scenario-starting-wood").fill("240");
  await page.getByTestId("scenario-forest-tile-count").fill("24");
  await page.getByTestId("scenario-forest-tile-wood").fill("95");
  await page.getByTestId("scenario-forest-regrowth-amount").fill("2");
  await page.getByTestId("scenario-forest-regrowth-interval").fill("11");
  await page.getByTestId("scenario-sawmill-output").fill("7");
  await page.getByTestId("scenario-sawmill-interval").fill("8");
  await page.getByTestId("scenario-sawmill-capacity").fill("31");
  await page.getByTestId("scenario-day-length-ticks").fill("875");
  await page.getByTestId("scenario-raid-rally-ticks").fill("43");
  await page.getByTestId("scenario-automatic-raids").uncheck();
  await page.getByTestId("scenario-pause-economy").uncheck();

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
      world: scenario.world,
      wave5: scenario.waves?.[4],
      advanced_health: scenario.raiders?.advanced?.health,
      arrow_build_cost: scenario.towers?.arrow?.build_cost,
    };
  });
  expect(savedScenario).toEqual({
    world: {
      starting_wood: 240,
      forest_tile_count: 24,
      forest_tile_wood: 95,
      forest_regrowth_amount: 2,
      forest_regrowth_interval_ticks: 11,
      sawmill_output: 7,
      sawmill_interval_ticks: 8,
      sawmill_local_wood_capacity: 31,
      day_length_ticks: 875,
      raid_rally_ticks: 43,
      automatic_raids: false,
      pause_economy_during_raids: false,
    },
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

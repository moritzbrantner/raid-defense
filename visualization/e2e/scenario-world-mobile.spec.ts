import { expect, test } from "@playwright/test";

test.use({ viewport: { width: 390, height: 844 } });

test("edits, starts, saves, reloads, and resumes explicit world settings on a phone", async ({ page }) => {
  await page.goto("/?screen=settings&section=scenario&scenarioView=world");

  await expect(page.getByTestId("scenario-world-editor")).toBeVisible();
  await expect(page.getByTestId("scenario-starting-wood")).toHaveValue("120");
  await expect(page.getByTestId("scenario-forest-tile-count")).toHaveValue("18");
  await expect(page.getByTestId("scenario-day-length-ticks")).toHaveValue("600");

  const worldNumberControls = [
    ["scenario-starting-wood", "240"],
    ["scenario-forest-tile-count", "24"],
    ["scenario-forest-tile-wood", "95"],
    ["scenario-forest-regrowth-amount", "2"],
    ["scenario-forest-regrowth-interval", "11"],
    ["scenario-sawmill-output", "7"],
    ["scenario-sawmill-interval", "8"],
    ["scenario-sawmill-capacity", "31"],
    ["scenario-day-length-ticks", "875"],
    ["scenario-raid-rally-ticks", "43"],
  ] as const;

  for (const [testId, value] of worldNumberControls) {
    const control = page.getByTestId(testId);
    await control.scrollIntoViewIfNeeded();
    await expect(control).toBeVisible();
    await control.fill(value);
    await expect(control).toHaveValue(value);
  }

  await page.getByTestId("scenario-automatic-raids").uncheck();
  await page.getByTestId("scenario-pause-economy").uncheck();
  await expect(page.getByTestId("scenario-automatic-raids")).not.toBeChecked();
  await expect(page.getByTestId("scenario-pause-economy")).not.toBeChecked();

  const pageFitsViewport = await page.evaluate(
    () => document.documentElement.scrollWidth <= window.innerWidth + 1,
  );
  expect(pageFitsViewport).toBe(true);

  await page.getByRole("button", { name: "Back" }).click();
  await page.getByTestId("new-game").click();
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("wood-value")).toHaveText("240");

  await page.getByTestId("return-to-menu").click();
  await expect(page.getByTestId("resume-game")).toBeEnabled();
  await page.reload();
  await expect(page.getByTestId("resume-game")).toBeEnabled();

  const savedWorld = await page.evaluate(() => {
    const raw = window.localStorage.getItem("raid-defense.save.v2");
    if (!raw) return null;
    return (JSON.parse(raw) as { scenario?: { world?: unknown } }).scenario?.world ?? null;
  });
  expect(savedWorld).toEqual({
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
  });

  await page.getByTestId("resume-game").click();
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("wood-value")).toHaveText("240");
});

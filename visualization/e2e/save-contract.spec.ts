import { expect, test } from "@playwright/test";

test("does not offer Resume for incompatible replay contracts", async ({ page }) => {
  await page.goto("/");

  await page.evaluate(() => {
    window.localStorage.setItem(
      "raid-defense.replay.v3",
      JSON.stringify({ version: 3, contract_version: 9 }),
    );
  });
  await page.reload();
  await expect(page.getByTestId("resume-game")).toBeDisabled();

  await page.evaluate(() => {
    window.localStorage.removeItem("raid-defense.replay.v3");
    window.localStorage.setItem(
      "raid-defense.save.v2",
      JSON.stringify({ version: 2, contract_version: 9 }),
    );
  });
  await page.reload();
  await expect(page.getByTestId("resume-game")).toBeDisabled();

  await page.evaluate(() => {
    window.localStorage.removeItem("raid-defense.save.v2");
    window.localStorage.setItem(
      "raid-defense.save.v1",
      JSON.stringify({ version: 1, contract_version: 9 }),
    );
  });
  await page.reload();
  await expect(page.getByTestId("resume-game")).toBeDisabled();
});

test("persists replay authority as timed player actions and keeps derived state separate", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByTestId("new-game").click();
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();

  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("cycle-timer")).toContainText("Rally ·");
  await page.waitForTimeout(350);
  await page.getByTestId("return-to-menu").click();

  const stored = await page.evaluate(() => {
    const replayRaw = window.localStorage.getItem("raid-defense.replay.v3");
    const summaryRaw = window.localStorage.getItem("raid-defense.save-summary.v1");
    return {
      replay: replayRaw ? JSON.parse(replayRaw) : null,
      summary: summaryRaw ? JSON.parse(summaryRaw) : null,
    };
  });

  expect(stored.replay).toMatchObject({
    version: 3,
    contract_version: 10,
    recorded_through_tick: expect.any(Number),
    actions: [
      {
        tick: expect.any(Number),
        sequence: 0,
        player_id: 0,
        command: { type: "start_wave" },
      },
    ],
  });
  expect(stored.replay.recorded_through_tick).toBeGreaterThanOrEqual(stored.replay.actions[0].tick);
  expect(stored.replay).not.toHaveProperty("checksum");
  expect(stored.replay).not.toHaveProperty("wave");
  expect(stored.replay).not.toHaveProperty("completed_waves");
  expect(stored.replay).not.toHaveProperty("entries");
  expect(stored.replay.actions.every((entry: any) => entry.command.type !== "advance_tick")).toBe(
    true,
  );

  expect(stored.summary).toMatchObject({
    checksum: expect.any(String),
    tick: stored.replay.recorded_through_tick,
    action_count: 1,
  });
});

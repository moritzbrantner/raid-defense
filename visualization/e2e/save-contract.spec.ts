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
    const integrityRaw = window.localStorage.getItem("raid-defense.replay-integrity.v1");
    const summaryRaw = window.localStorage.getItem("raid-defense.save-summary.v1");
    return {
      replay: replayRaw ? JSON.parse(replayRaw) : null,
      integrity: integrityRaw ? JSON.parse(integrityRaw) : null,
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

  expect(stored.integrity).toMatchObject({
    version: 1,
    replay_version: 3,
    contract_version: 10,
    seed: stored.replay.seed,
    recorded_through_tick: stored.replay.recorded_through_tick,
    action_count: 1,
    checksum: expect.any(String),
  });
  expect(stored.summary).toMatchObject({
    checksum: stored.integrity.checksum,
    tick: stored.replay.recorded_through_tick,
    action_count: 1,
  });
});

test("fails closed when replay integrity does not match reconstructed state", async ({ page }) => {
  await page.goto("/");
  await page.getByTestId("new-game").click();
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("cycle-timer")).toContainText("Rally ·");
  await page.getByTestId("return-to-menu").click();

  await page.reload();
  await expect(page.getByTestId("resume-game")).toBeEnabled();

  const beforeTamper = await page.evaluate(() => {
    const summaryRaw = window.localStorage.getItem("raid-defense.save-summary.v1");
    if (!summaryRaw) throw new Error("expected saved-game summary");
    return JSON.parse(summaryRaw) as { checksum: string };
  });

  await page.evaluate(() => {
    const raw = window.localStorage.getItem("raid-defense.replay-integrity.v1");
    if (!raw) throw new Error("expected replay integrity receipt");
    const receipt = JSON.parse(raw) as { checksum: string };
    receipt.checksum = "corrupted-checksum";
    window.localStorage.setItem("raid-defense.replay-integrity.v1", JSON.stringify(receipt));
  });

  await page.getByTestId("resume-game").click();
  await page.waitForTimeout(1_000);

  const afterRejectedResume = await page.evaluate(() => {
    const integrityRaw = window.localStorage.getItem("raid-defense.replay-integrity.v1");
    const summaryRaw = window.localStorage.getItem("raid-defense.save-summary.v1");
    if (!integrityRaw || !summaryRaw) throw new Error("expected replay persistence");
    return {
      integrity: JSON.parse(integrityRaw) as { checksum: string },
      summary: JSON.parse(summaryRaw) as { checksum: string },
    };
  });

  expect(afterRejectedResume.integrity.checksum).toBe("corrupted-checksum");
  expect(afterRejectedResume.summary.checksum).toBe(beforeTamper.checksum);
  expect(afterRejectedResume.summary.checksum).not.toBe(afterRejectedResume.integrity.checksum);
});

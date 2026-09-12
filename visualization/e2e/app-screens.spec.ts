import { expect, test } from "@playwright/test";

async function openGame(page: import("@playwright/test").Page) {
  await page.goto("/");
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("day-value")).toHaveText("0");
}

test("plays the opening fort claim and reinforcement flow", async ({ page }) => {
  await openGame(page);

  await expect(page.getByTestId("treasury-value")).toHaveText("100");
  const initialChecksum = await page.getByTestId("checksum").textContent();

  await page.getByTestId("build-fort").click();
  await expect(page.getByTestId("event-feedback")).toContainText("fort upgraded to level 1");
  await expect(page.getByTestId("treasury-value")).toHaveText("80");

  await page.getByTestId("province-1").click();
  await page.getByTestId("claim-province").click();
  await expect(page.getByTestId("event-feedback")).toContainText("North March joined the frontier");
  await expect(page.getByTestId("province-1")).toContainText("0 garrison · fort 0");

  await page.getByTestId("recruit-five").click();
  await expect(page.getByTestId("event-feedback")).toContainText("5 soldiers reinforced North March");
  await expect(page.getByTestId("province-1")).toContainText("5 garrison");

  await expect(page.getByTestId("checksum")).not.toHaveText(initialChecksum ?? "");
});

test("surfaces the first seeded raid after three days", async ({ page }) => {
  await openGame(page);

  await page.getByTestId("advance-day").click();
  await page.getByTestId("advance-day").click();
  await page.getByTestId("advance-day").click();

  await expect(page.getByTestId("day-value")).toHaveText("3");
  await expect(page.getByTestId("raid-warning")).toContainText("Raid → Capital");
  await expect(page.getByTestId("event-feedback")).toContainText("Raid sighted against Capital");
});

test("rejected frontier claim preserves authoritative checksum", async ({ page }) => {
  await openGame(page);

  await page.getByTestId("province-2").click();
  const checksumBefore = await page.getByTestId("checksum").textContent();
  const treasuryBefore = await page.getByTestId("treasury-value").textContent();

  await page.getByTestId("claim-province").click();

  await expect(page.getByTestId("event-feedback")).toContainText(
    "adjacent controlled fort with stable control",
  );
  await expect(page.getByTestId("checksum")).toHaveText(checksumBefore ?? "");
  await expect(page.getByTestId("treasury-value")).toHaveText(treasuryBefore ?? "");
});

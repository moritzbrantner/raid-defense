import { expect, test } from "@playwright/test";

async function openGame(page: import("@playwright/test").Page) {
  await page.goto("/");
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("world-3d")).toBeVisible();
  await expect(page.getByTestId("gold-value")).toHaveText("120");
}

test("builds a guard tower on the authoritative grid", async ({ page }) => {
  await openGame(page);

  const initialChecksum = await page.getByTestId("checksum").textContent();
  await page.getByTestId("cell-x").fill("2");
  await page.getByTestId("cell-z").fill("2");
  await page.getByTestId("build-tower").click();

  await expect(page.getByTestId("event-feedback")).toContainText("Guard tower built at 2, 2");
  await expect(page.getByTestId("gold-value")).toHaveText("95");
  await expect(page.getByTestId("tower-count")).toHaveText("1");
  await expect(page.getByTestId("checksum")).not.toHaveText(initialChecksum ?? "");
});

test("runs a real-time wave from the map edges", async ({ page }) => {
  await openGame(page);

  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("event-feedback")).toContainText("Wave 1 started from all four edges");
  await expect(page.getByTestId("wave-value")).toHaveText("1");

  await expect
    .poll(async () => Number(await page.getByTestId("tick-value").textContent()))
    .toBeGreaterThan(0);
});

test("rejects building on the protected town without mutating state", async ({ page }) => {
  await openGame(page);

  await page.getByTestId("cell-x").fill("8");
  await page.getByTestId("cell-z").fill("6");
  const checksumBefore = await page.getByTestId("checksum").textContent();
  const goldBefore = await page.getByTestId("gold-value").textContent();

  await page.getByTestId("build-tower").click();

  await expect(page.getByTestId("event-feedback")).toContainText("reserved for the town or an edge spawn gate");
  await expect(page.getByTestId("checksum")).toHaveText(checksumBefore ?? "");
  await expect(page.getByTestId("gold-value")).toHaveText(goldBefore ?? "");
  await expect(page.getByTestId("tower-count")).toHaveText("0");
});

import { expect, test } from "@playwright/test";

async function openGame(page: import("@playwright/test").Page) {
  await page.goto("/");
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("world-3d")).toBeVisible();
  await expect(page.getByTestId("gold-value")).toHaveText("120");
}

async function selectCell(page: import("@playwright/test").Page, x: number, z: number) {
  await page.getByTestId("cell-x").fill(String(x));
  await page.getByTestId("cell-z").fill(String(z));
}

test("builds distinct arrow and cannon towers on the authoritative grid", async ({ page }) => {
  await openGame(page);

  const initialChecksum = await page.getByTestId("checksum").textContent();
  await selectCell(page, 2, 2);
  await page.getByTestId("build-arrow-tower").click();
  await expect(page.getByTestId("event-feedback")).toContainText("Arrow tower built at 2, 2");

  await selectCell(page, 3, 2);
  await page.getByTestId("build-cannon-tower").click();
  await expect(page.getByTestId("event-feedback")).toContainText("Cannon tower built at 3, 2");

  await expect(page.getByTestId("gold-value")).toHaveText("50");
  await expect(page.getByTestId("tower-count")).toHaveText("2");
  await expect(page.getByTestId("checksum")).not.toHaveText(initialChecksum ?? "");
});

test("upgrades the selected tower through the Rust command boundary", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 2, 2);
  await page.getByTestId("build-arrow-tower").click();
  await expect(page.getByTestId("selected-tower")).toContainText("Arrow tower · L1");

  await page.getByTestId("upgrade-tower").click();

  await expect(page.getByTestId("event-feedback")).toContainText("upgraded to level 2 for 20g");
  await expect(page.getByTestId("selected-tower")).toContainText("Arrow tower · L2");
  await expect(page.getByTestId("gold-value")).toHaveText("75");
});

test("runs a live wave with visible authoritative projectile entities", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 7, 2);
  await page.getByTestId("build-arrow-tower").click();
  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("event-feedback")).toContainText("Wave 1 started from all four edges");
  await expect(page.getByTestId("wave-value")).toHaveText("1");

  await expect
    .poll(async () => Number(await page.getByTestId("tick-value").textContent()))
    .toBeGreaterThan(0);
  await expect
    .poll(async () => Number(await page.getByTestId("projectile-count").textContent()))
    .toBeGreaterThan(0);
});

test("rejects building on the protected town without mutating state", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 8, 6);
  const checksumBefore = await page.getByTestId("checksum").textContent();
  const goldBefore = await page.getByTestId("gold-value").textContent();

  await page.getByTestId("build-cannon-tower").click();

  await expect(page.getByTestId("event-feedback")).toContainText(
    "reserved for the town or an edge spawn gate",
  );
  await expect(page.getByTestId("checksum")).toHaveText(checksumBefore ?? "");
  await expect(page.getByTestId("gold-value")).toHaveText(goldBefore ?? "");
  await expect(page.getByTestId("tower-count")).toHaveText("0");
});

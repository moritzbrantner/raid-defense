import { expect, test } from "@playwright/test";

async function openGame(page: import("@playwright/test").Page) {
  await page.goto("/");
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("world-3d")).toBeVisible();
  await expect(page.getByTestId("wood-value")).toHaveText("120");
  await expect(page.getByTestId("people-value")).toHaveText("2/2");
  await expect(page.getByTestId("sawmill-count")).toHaveText("0");
  await expect(page.getByTestId("completed-waves-value")).toHaveText("0");
}

async function selectCell(page: import("@playwright/test").Page, x: number, z: number) {
  await page.getByTestId("cell-x").fill(String(x));
  await page.getByTestId("cell-z").fill(String(z));
}

test("sawmill wood waits for real people to carry it back to the Town Hall", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 2, 2);
  await page.getByTestId("build-sawmill").click();

  await expect(page.getByTestId("event-feedback")).toContainText("must be carried back");
  await expect(page.getByTestId("sawmill-count")).toHaveText("1");
  await expect(page.getByTestId("wood-value")).toHaveText("80");
  await expect(page.getByTestId("selected-building")).toContainText("local wood 0/24");

  await expect
    .poll(async () => Number(await page.getByTestId("hauling-value").textContent()), {
      timeout: 8_000,
    })
    .toBeGreaterThan(0);

  await expect
    .poll(async () => Number(await page.getByTestId("wood-value").textContent()), {
      timeout: 12_000,
    })
    .toBeGreaterThan(80);
});

test("Houses are visibly locked until ten completed waves", async ({ page }) => {
  await openGame(page);

  await expect(page.getByTestId("house-lock-state")).toContainText(
    "Houses unlock after 10 completed waves (0/10)",
  );
  await expect(page.getByTestId("build-house")).toBeDisabled();
  await expect(page.getByTestId("build-house")).toContainText("unlocks after 10 waves");
});

test("spends Town Hall wood on distinct tower archetypes", async ({ page }) => {
  await openGame(page);

  const initialChecksum = await page.getByTestId("checksum").textContent();
  await selectCell(page, 2, 2);
  await page.getByTestId("build-arrow-tower").click();
  await expect(page.getByTestId("event-feedback")).toContainText("Arrow tower built at 2, 2");

  await selectCell(page, 3, 2);
  await page.getByTestId("build-cannon-tower").click();
  await expect(page.getByTestId("event-feedback")).toContainText("Cannon tower built at 3, 2");

  await expect(page.getByTestId("wood-value")).toHaveText("50");
  await expect(page.getByTestId("tower-count")).toHaveText("2");
  await expect(page.getByTestId("checksum")).not.toHaveText(initialChecksum ?? "");
});

test("upgrades the selected tower by spending wood", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 2, 2);
  await page.getByTestId("build-arrow-tower").click();
  await expect(page.getByTestId("selected-building")).toContainText("Arrow tower · L1");

  await page.getByTestId("upgrade-tower").click();

  await expect(page.getByTestId("event-feedback")).toContainText("upgraded to level 2 for 20 wood");
  await expect(page.getByTestId("selected-building")).toContainText("Arrow tower · L2");
  await expect(page.getByTestId("wood-value")).toHaveText("75");
});

test("raiders steal stored wood when they reach the Town Hall", async ({ page }) => {
  await openGame(page);

  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("wave-value")).toHaveText("1");

  await expect
    .poll(async () => Number(await page.getByTestId("wood-value").textContent()), {
      timeout: 8_000,
    })
    .toBeLessThan(120);
  await expect(page.getByTestId("event-feedback")).toContainText("wood stolen");
});

test("runs a defended raid with authoritative projectile entities", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 7, 2);
  await page.getByTestId("build-arrow-tower").click();
  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("wave-value")).toHaveText("1");

  await expect
    .poll(async () => {
      const text = await page.getByTestId("projectile-count").textContent();
      return Number(text?.replace("Projectiles ", "") ?? "0");
    })
    .toBeGreaterThan(0);
});

test("rejects building on the protected Town Hall without mutating state", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 8, 6);
  const checksumBefore = await page.getByTestId("checksum").textContent();
  const woodBefore = await page.getByTestId("wood-value").textContent();

  await page.getByTestId("build-sawmill").click();

  await expect(page.getByTestId("event-feedback")).toContainText(
    "reserved for the Town Hall or an edge spawn gate",
  );
  await expect(page.getByTestId("checksum")).toHaveText(checksumBefore ?? "");
  await expect(page.getByTestId("wood-value")).toHaveText(woodBefore ?? "");
  await expect(page.getByTestId("sawmill-count")).toHaveText("0");
});

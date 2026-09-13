import { expect, test } from "@playwright/test";

async function openGame(page: import("@playwright/test").Page) {
  await page.goto("/");
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("world-3d")).toBeVisible();
  await expect(page.getByTestId("wood-value")).toHaveText("120");
  await expect(page.getByTestId("people-value")).toHaveText("2/2");
  await expect(page.getByTestId("forest-count")).toHaveText("18");
  await expect(page.getByTestId("storage-count")).toHaveText("0");
  await expect(page.getByTestId("sawmill-count")).toHaveText("0");
  await expect(page.getByTestId("construction-count")).toHaveText("0");
  await expect(page.getByTestId("completed-waves-value")).toHaveText("0");
  await expect(page.getByTestId("cycle-timer")).toContainText("Day ·");
  await expect(page.locator("footer")).toContainText("Map 21×21");
}

async function selectCell(page: import("@playwright/test").Page, x: number, z: number) {
  await page.getByTestId("cell-x").fill(String(x));
  await page.getByTestId("cell-z").fill(String(z));
}

async function waitForTower(page: import("@playwright/test").Page) {
  await expect(page.getByTestId("tower-count")).toHaveText("1", { timeout: 25_000 });
  await expect(page.getByTestId("construction-count")).toHaveText("0");
}

test("cycle timer counts down through daytime and shows nighttime", async ({ page }) => {
  await openGame(page);

  const initialTimer = await page.getByTestId("cycle-timer").textContent();
  await expect
    .poll(async () => page.getByTestId("cycle-timer").textContent(), { timeout: 3_000 })
    .not.toBe(initialTimer);

  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("cycle-timer")).toHaveText("Night · Wave 1");
});

test("seeded forest feeds a sawmill and people carry harvested wood into storage", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 2, 2);
  await page.getByTestId("build-sawmill").click();

  await expect(page.getByTestId("event-feedback")).toContainText("harvests nearby forest");
  await expect(page.getByTestId("sawmill-count")).toHaveText("1");
  await expect(page.getByTestId("wood-value")).toHaveText("80");
  await expect(page.getByTestId("selected-building")).toContainText("local wood 0/24");

  await expect
    .poll(async () => Number(await page.getByTestId("hauling-value").textContent()), {
      timeout: 10_000,
    })
    .toBeGreaterThan(0);

  await expect
    .poll(async () => Number(await page.getByTestId("wood-value").textContent()), {
      timeout: 15_000,
    })
    .toBeGreaterThan(80);
});

test("storage houses extend distributed settlement storage", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 3, 2);
  await page.getByTestId("build-storage-house").click();
  await expect(page.getByTestId("event-feedback")).toContainText("Storage house built at 3, 2");
  await expect(page.getByTestId("storage-count")).toHaveText("1");
  await expect(page.getByTestId("wood-value")).toHaveText("70");
  await expect(page.getByTestId("selected-building")).toContainText("Storage house · 0/160 wood");
});

test("tower placement creates a construction site until workers deliver material", async ({ page }) => {
  await openGame(page);

  const initialChecksum = await page.getByTestId("checksum").textContent();
  await selectCell(page, 2, 2);
  await page.getByTestId("build-arrow-tower").click();

  await expect(page.getByTestId("event-feedback")).toContainText("Workers must haul 25 wood");
  await expect(page.getByTestId("construction-count")).toHaveText("1");
  await expect(page.getByTestId("tower-count")).toHaveText("0");
  await expect
    .poll(async () => Number(await page.getByTestId("wood-value").textContent()), {
      timeout: 5_000,
    })
    .toBeLessThan(120);
  await expect(page.getByTestId("selected-building")).toContainText("construction");

  await waitForTower(page);
  await expect(page.getByTestId("selected-building")).toContainText("Arrow tower · L1");
  await expect
    .poll(async () => page.getByTestId("wood-value").textContent(), { timeout: 15_000 })
    .toBe("95");
  await expect(page.getByTestId("checksum")).not.toHaveText(initialChecksum ?? "");
});

test("upgrades only after the selected tower has been physically constructed", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 2, 2);
  await page.getByTestId("build-arrow-tower").click();
  await expect(page.getByTestId("upgrade-tower")).toBeDisabled();
  await waitForTower(page);
  await expect(page.getByTestId("upgrade-tower")).toBeEnabled();

  await page.getByTestId("upgrade-tower").click();

  await expect(page.getByTestId("event-feedback")).toContainText("upgraded to level 2 for 20 wood");
  await expect(page.getByTestId("selected-building")).toContainText("Arrow tower · L2");
  await expect(page.getByTestId("wood-value")).toHaveText("75");
});

test("Houses are visibly locked until ten completed waves", async ({ page }) => {
  await openGame(page);

  await expect(page.getByTestId("house-lock-state")).toContainText(
    "Houses unlock after 10 completed waves (0/10)",
  );
  await expect(page.getByTestId("build-house")).toBeDisabled();
  await expect(page.getByTestId("build-house")).toContainText("unlocks after 10 waves");
});

test("raiders steal stored wood when they reach settlement storage", async ({ page }) => {
  await openGame(page);

  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("wave-value")).toHaveText("1");

  await expect
    .poll(async () => Number(await page.getByTestId("wood-value").textContent()), {
      timeout: 12_000,
    })
    .toBeLessThan(120);
  await expect(page.getByTestId("event-feedback")).toContainText("wood stolen");
});

test("runs a defended raid with authoritative projectile entities", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 7, 2);
  await page.getByTestId("build-arrow-tower").click();
  await waitForTower(page);
  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("wave-value")).toHaveText("1");

  await expect
    .poll(async () => {
      const text = await page.getByTestId("projectile-count").textContent();
      return Number(text?.replace("Projectiles ", "") ?? "0");
    }, { timeout: 12_000 })
    .toBeGreaterThan(0);
});

test("rejects building on the protected Town Hall without mutating state", async ({ page }) => {
  await openGame(page);

  await selectCell(page, 10, 10);
  const woodBefore = await page.getByTestId("wood-value").textContent();

  await page.getByTestId("build-storage-house").click();

  await expect(page.getByTestId("event-feedback")).toContainText(
    "reserved for the Town Hall or an edge spawn gate",
  );
  await expect(page.getByTestId("wood-value")).toHaveText(woodBefore ?? "");
  await expect(page.getByTestId("storage-count")).toHaveText("0");
});

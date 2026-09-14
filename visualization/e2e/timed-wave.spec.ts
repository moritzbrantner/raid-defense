import { expect, test } from "@playwright/test";

test("spawns a raid over authoritative simulation time", async ({ page }) => {
  await page.goto("/?screen=game&new=1&seed=24301");
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();

  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("wave-value")).toHaveText("1");
  await expect(page.getByTestId("cycle-timer")).toHaveText("Night · Wave 1");
  await expect(page.getByTestId("start-wave")).toBeDisabled();

  const initialRaiders = Number(await page.getByTestId("raider-count").textContent());
  expect(initialRaiders).toBe(1);

  await expect
    .poll(async () => Number(await page.getByTestId("raider-count").textContent()), {
      timeout: 4_000,
    })
    .toBeGreaterThan(1);

  await expect(page.getByTestId("cycle-timer")).toHaveText("Night · Wave 1");
  await expect(page.getByTestId("start-wave")).toBeDisabled();
});

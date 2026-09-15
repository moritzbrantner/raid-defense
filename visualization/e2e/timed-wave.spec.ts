import { expect, test } from "@playwright/test";

test("rallies workers before spawning a raid over authoritative simulation time", async ({ page }) => {
  await page.goto("/?screen=game&new=1&seed=24301");
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();

  await page.getByTestId("start-wave").click();
  await expect(page.getByTestId("cycle-timer")).toContainText("Rally ·");
  await expect(page.getByTestId("wave-value")).toHaveText("0");
  await expect(page.getByTestId("raider-count")).toHaveText("0");
  await expect(page.getByTestId("start-wave")).toBeDisabled();

  await expect(page.getByTestId("cycle-timer")).toHaveText("Night · Wave 1", {
    timeout: 12_000,
  });
  await expect(page.getByTestId("wave-value")).toHaveText("1");
  await expect
    .poll(async () => Number(await page.getByTestId("raider-count").textContent()), {
      timeout: 5_000,
    })
    .toBeGreaterThan(0);

  await expect
    .poll(async () => Number(await page.getByTestId("raider-count").textContent()), {
      timeout: 5_000,
    })
    .toBeGreaterThan(1);

  await expect(page.getByTestId("cycle-timer")).toHaveText("Night · Wave 1");
  await expect(page.getByTestId("start-wave")).toBeDisabled();
});

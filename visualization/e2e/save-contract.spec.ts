import { expect, test } from "@playwright/test";

test("does not offer Resume for pre-rally save contracts", async ({ page }) => {
  await page.goto("/");

  await page.evaluate(() => {
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

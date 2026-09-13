import { expect, test } from "@playwright/test";

test("opens an isolated presentation-only field guide for the core game systems", async ({ page }) => {
  await page.goto("/?screen=game&new=1&seed=24301");
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();

  const launcher = page.getByTestId("open-wiki");
  await launcher.click();

  const wiki = page.getByTestId("game-wiki");
  const close = page.getByTestId("close-wiki");
  await expect(wiki).toBeVisible();
  await expect(wiki).toContainText("Raid Defense field guide");
  await expect(wiki).toContainText("Forests, sawmills, and storage");
  await expect(wiki).toContainText("Workers and construction");
  await expect(wiki).toContainText("Raiders");
  await expect(wiki).toContainText("Deterministic simulation");
  await expect(close).toBeFocused();

  await page.keyboard.press("Tab");
  await expect(close).toBeFocused();

  await page.keyboard.press("Escape");
  await expect(wiki).toBeHidden();
  await expect(launcher).toBeFocused();
});

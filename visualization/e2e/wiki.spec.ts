import { expect, test } from "@playwright/test";

test("opens a presentation-only field guide for the core game systems", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();

  await page.getByTestId("open-wiki").click();

  const wiki = page.getByTestId("game-wiki");
  await expect(wiki).toBeVisible();
  await expect(wiki).toContainText("Raid Defense field guide");
  await expect(wiki).toContainText("Forests, sawmills, and storage");
  await expect(wiki).toContainText("Workers and construction");
  await expect(wiki).toContainText("Raiders");
  await expect(wiki).toContainText("Deterministic simulation");

  await page.keyboard.press("Escape");
  await expect(wiki).toBeHidden();
});

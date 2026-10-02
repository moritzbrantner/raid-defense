import { expect, test } from "@playwright/test";

test("language selection is URL-backed and survives navigation and reload", async ({ page }) => {
  await page.goto("/?screen=settings&lang=en");

  await expect(page.getByRole("heading", { name: "Settings" })).toBeVisible();
  await expect(page.getByTestId("setting-language")).toHaveValue("en");

  await page.getByTestId("setting-language").selectOption("de");

  await expect(page).toHaveURL(/screen=settings.*lang=de|lang=de.*screen=settings/);
  await expect(page.locator("html")).toHaveAttribute("lang", "de");
  await expect(page.getByRole("heading", { name: "Einstellungen" })).toBeVisible();

  await page.getByTestId("settings-back").click();
  await expect(page.getByTestId("new-game")).toHaveText("Neues Spiel");
  await expect(page).toHaveURL(/screen=menu.*lang=de|lang=de.*screen=menu/);

  await page.reload();
  await expect(page.getByTestId("new-game")).toHaveText("Neues Spiel");
  await expect(page.locator("html")).toHaveAttribute("lang", "de");
});

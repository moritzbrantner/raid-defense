import { expect, test } from "@playwright/test";

test("starts a new game and resumes saved authoritative progress", async ({ page }) => {
  await page.goto("/");

  await expect(page.getByTestId("start-menu")).toBeVisible();
  await expect(page.getByTestId("resume-game")).toBeDisabled();

  await page.getByTestId("new-game").click();
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();

  await page.getByTestId("cell-x").fill("3");
  await page.getByTestId("cell-z").fill("2");
  await page.getByTestId("build-storage-house").click();
  await expect(page.getByTestId("storage-count")).toHaveText("1");
  await expect(page.getByTestId("event-feedback")).toContainText("Storage house built at 3, 2");

  await page.getByTestId("return-to-menu").click();
  await expect(page.getByTestId("start-menu")).toBeVisible();
  await expect(page.getByTestId("resume-game")).toBeEnabled();

  await page.reload();
  await expect(page.getByTestId("start-menu")).toBeVisible();
  await expect(page.getByTestId("resume-game")).toBeEnabled();

  await page.getByTestId("resume-game").click();
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("storage-count")).toHaveText("1");
  await expect(page.getByTestId("wood-value")).toHaveText("70");
});

test("opens URL-addressable settings and persists presentation preferences", async ({ page }) => {
  await page.goto("/");

  await page.getByTestId("open-settings").click();
  await expect(page.getByTestId("settings-screen")).toBeVisible();
  await expect(page).toHaveURL(/screen=settings/);

  const touchHints = page.getByTestId("setting-touch-hints");
  const reduceMotion = page.getByTestId("setting-reduce-motion");
  const compactStatus = page.getByTestId("setting-compact-status");
  await expect(touchHints).toBeChecked();
  await expect(reduceMotion).not.toBeChecked();
  await expect(compactStatus).not.toBeChecked();

  await touchHints.uncheck();
  await reduceMotion.check();
  await compactStatus.check();
  await page.reload();

  await expect(page.getByTestId("settings-screen")).toBeVisible();
  await expect(touchHints).not.toBeChecked();
  await expect(reduceMotion).toBeChecked();
  await expect(compactStatus).toBeChecked();
});

import { expect, test } from "@playwright/test";

test.use({
  viewport: { width: 390, height: 844 },
  hasTouch: true,
});

test("keeps core controls reachable and touch-sized on a phone", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("raid-defense-game")).toBeVisible();
  await expect(page.getByTestId("mobile-world-hint")).toContainText("tap a tile", {
    ignoreCase: true,
  });

  const dock = page.getByTestId("mobile-command-dock");
  await expect(dock).toBeVisible();
  await expect(dock).toHaveCSS("position", "fixed");

  const touchTargets = [
    page.getByTestId("start-wave"),
    page.getByTestId("cell-west"),
    page.getByTestId("cell-north"),
    page.getByTestId("cell-south"),
    page.getByTestId("cell-east"),
    page.getByTestId("build-sawmill"),
    page.getByTestId("build-storage-house"),
    page.getByTestId("build-arrow-tower"),
  ];
  for (const target of touchTargets) {
    const box = await target.boundingBox();
    expect(box).not.toBeNull();
    expect(box?.height ?? 0).toBeGreaterThanOrEqual(44);
  }

  await expect(page.getByTestId("mobile-selected-cell")).toHaveText("2,2");
  await page.getByTestId("cell-east").click();
  await expect(page.getByTestId("mobile-selected-cell")).toHaveText("3,2");
  await expect(page.getByTestId("cell-x")).toHaveValue("3");

  await page.getByTestId("build-storage-house").click();
  await expect(page.getByTestId("storage-count")).toHaveText("1");
  await expect(page.getByTestId("event-feedback")).toContainText("Storage house built at 3, 2");

  const canvasTouchAction = await page
    .locator(".battlefield canvas")
    .evaluate((element) => getComputedStyle(element).touchAction);
  expect(canvasTouchAction).toBe("none");

  const pageFitsViewport = await page.evaluate(
    () => document.documentElement.scrollWidth <= window.innerWidth + 1,
  );
  expect(pageFitsViewport).toBe(true);

  const guide = page.getByTestId("open-wiki");
  const [guideBox, dockBox] = await Promise.all([guide.boundingBox(), dock.boundingBox()]);
  expect(guideBox).not.toBeNull();
  expect(dockBox).not.toBeNull();
  expect((guideBox?.y ?? 0) + (guideBox?.height ?? 0)).toBeLessThanOrEqual(dockBox?.y ?? 0);
});

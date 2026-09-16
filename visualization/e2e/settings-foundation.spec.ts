import { expect, test } from "@playwright/test";

test("preserves presentation edits made while shared settings initialize", async ({ page }) => {
  let releaseFoundation!: () => void;
  let markFoundationRequested!: () => void;
  const foundationGate = new Promise<void>((resolve) => {
    releaseFoundation = resolve;
  });
  const foundationRequested = new Promise<void>((resolve) => {
    markFoundationRequested = resolve;
  });

  await page.route("**/settings-browser.js", async (route) => {
    markFoundationRequested();
    await foundationGate;
    await route.continue();
  });

  await page.goto("/?screen=settings&section=presentation");
  await foundationRequested;

  const touchHints = page.getByTestId("setting-touch-hints");
  await expect(touchHints).toBeChecked();
  await touchHints.uncheck();
  await expect(touchHints).not.toBeChecked();

  releaseFoundation();

  await expect
    .poll(
      () =>
        page.evaluate(() => {
          const raw = window.localStorage.getItem("raid-defense.settings.user.v2");
          if (!raw) return null;
          const snapshot = JSON.parse(raw) as {
            overrides?: Record<string, { value?: unknown }>;
          };
          return snapshot.overrides?.["presentation.show_touch_hints"]?.value;
        }),
      { timeout: 10_000 },
    )
    .toBe(false);

  await expect(touchHints).not.toBeChecked();
});

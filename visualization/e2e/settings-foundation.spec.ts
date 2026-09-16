import { expect, test, type Page } from "@playwright/test";

function presentationSwitch(page: Page, settingId: string) {
  return page.locator(`[data-setting-id="${settingId}"] [data-slot="switch"]`);
}

test("preserves presentation edits made while shared settings initialize", async ({ page }) => {
  let releaseFoundation!: () => void;
  let markFoundationRequested!: () => void;
  const foundationGate = new Promise<void>((resolve) => {
    releaseFoundation = resolve;
  });
  const foundationRequested = new Promise<void>((resolve) => {
    markFoundationRequested = resolve;
  });

  await page.route("**/*settings_wasm_bg.wasm*", async (route) => {
    markFoundationRequested();
    await foundationGate;
    await route.continue();
  });

  await page.goto("/?screen=settings&section=presentation");
  await foundationRequested;

  const touchHints = presentationSwitch(page, "presentation.show_touch_hints");
  await expect(touchHints).toHaveAttribute("aria-checked", "true");
  await touchHints.click();
  await expect(touchHints).toHaveAttribute("aria-checked", "false");

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

  await expect(touchHints).toHaveAttribute("aria-checked", "false");
});

test("keeps presentation controls usable when browser storage rejects writes", async ({ page }) => {
  const pageErrors: string[] = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));
  await page.addInitScript(() => {
    Storage.prototype.setItem = function setItem() {
      throw new DOMException("Storage disabled for test", "QuotaExceededError");
    };
  });

  await page.goto("/?screen=settings&section=presentation");

  const touchHints = presentationSwitch(page, "presentation.show_touch_hints");
  const reduceMotion = presentationSwitch(page, "accessibility.reduce_motion");
  await expect(touchHints).toHaveAttribute("aria-checked", "true");
  await expect(reduceMotion).toHaveAttribute("aria-checked", "false");

  await touchHints.click();
  await reduceMotion.click();

  await expect(touchHints).toHaveAttribute("aria-checked", "false");
  await expect(reduceMotion).toHaveAttribute("aria-checked", "true");
  expect(pageErrors).toEqual([]);
});

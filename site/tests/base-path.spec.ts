import { expect, test } from "@playwright/test";

// Smoke test for GitHub Pages subpath serving. Runs only when the harness
// serves a base-path build:
//   E2E_CMD=preview E2E_BASE=/dotmax/ BASE_PATH_E2E=1 node scripts/run-e2e.mjs
test.skip(!process.env.BASE_PATH_E2E, "base-path smoke runs only against a --base build");

test("site works when served from a subpath", async ({ page }) => {
  const failures: string[] = [];
  page.on("response", (response) => {
    if (response.status() >= 400) failures.push(`${response.status()} ${response.url()}`);
  });

  await page.goto("./");

  // Hero showcase fetches hero.json through the base-aware loader.
  await expect(page.locator(".hero-showcase-output")).not.toBeEmpty();
  await expect(page.locator(".hero-showcase-output span[style*='color']").first()).toBeAttached();

  // The style browser loads the catalog index (644 cards gated to 12).
  await page.locator("#loading-bars").scrollIntoViewIfNeeded();
  await expect(page.locator(".loading-bar-card")).toHaveCount(12);
  await expect(page.locator(".loading-bar-output span[style*='color']").first()).toBeAttached();

  // Relative in-page fetch resolves under the subpath.
  const catalogOk = await page.evaluate(async () => (await fetch("catalog/index.json")).ok);
  expect(catalogOk).toBe(true);

  // llms.txt serves under the subpath.
  const llms = await page.request.get("llms.txt");
  expect(llms.ok()).toBe(true);

  // Gallery images actually load (no root-absolute 404s). They're lazy, so
  // scroll them into view and poll for decode.
  const docImg = page.locator(".doc-card-visual img").first();
  await docImg.scrollIntoViewIfNeeded();
  await expect
    .poll(async () => docImg.evaluate((node) => (node as HTMLImageElement).naturalWidth), {
      timeout: 5000,
    })
    .toBeGreaterThan(0);

  expect(failures, `unexpected 4xx/5xx responses:\n${failures.join("\n")}`).toEqual([]);
});

import { expect, test } from "@playwright/test";
import fs from "node:fs";

const heroPack = JSON.parse(
  fs.readFileSync(new URL("../public/catalog/hero.json", import.meta.url), "utf-8"),
) as {
  theme: string;
  width: number;
  height: number;
  fps: number;
  frames_per_style: number;
  styles: Array<{ id: string; frames: unknown[]; source: string }>;
};

const catalogIndex = JSON.parse(
  fs.readFileSync(new URL("../public/catalog/index.json", import.meta.url), "utf-8"),
) as { styles: Array<{ id: string }> };

test("hero pack is curated, colored, and every pick exists in the catalog", () => {
  expect(heroPack.theme).toBe("hero");
  expect(heroPack.width).toBe(44);
  expect(heroPack.height).toBe(4);
  expect(heroPack.fps).toBe(12);
  expect(heroPack.styles.length).toBeGreaterThanOrEqual(8);
  expect(heroPack.styles.length).toBeLessThanOrEqual(12);
  const ids = new Set(catalogIndex.styles.map((s) => s.id));
  for (const style of heroPack.styles) {
    expect(ids.has(style.id), `hero pick ${style.id} missing from index`).toBe(true);
    expect(style.frames.length).toBe(heroPack.frames_per_style);
    // Hero strips source; the dialog loads the real theme pack instead.
    expect(style.source).toBe("");
  }
});

test("hero showcase plays live frames and opens the style dialog", async ({ page }) => {
  await page.goto("/");

  const showcase = page.locator(".hero-showcase");
  await expect(showcase).toBeVisible();

  // A real style is shown (not the skeleton) with non-empty frames.
  await expect(showcase).toHaveAttribute("data-hero-style", /.+\/.+/);
  await expect(page.locator(".hero-showcase-output")).not.toBeEmpty();

  // The shared ticker advances the hero frame counter.
  const initialFrame = await showcase.getAttribute("data-hero-frame");
  await page.waitForTimeout(350);
  await expect(showcase).not.toHaveAttribute("data-hero-frame", initialFrame ?? "");

  // Colors reach the DOM as inline spans.
  await expect(page.locator(".hero-showcase-output span[style*='color']").first()).toBeAttached();

  // Click-through opens the detail dialog with all four copy tabs.
  await page.locator(".hero-showcase-body").click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(dialog.locator(".style-dialog-tab")).toHaveCount(4);
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
});

test("hero showcase dots switch the shown style", async ({ page }) => {
  await page.goto("/");
  const showcase = page.locator(".hero-showcase");
  await expect(showcase).toHaveAttribute("data-hero-style", heroPack.styles[0].id);

  await page.locator(".hero-showcase-dot").nth(2).click();
  await expect(showcase).toHaveAttribute("data-hero-style", heroPack.styles[2].id);
});

test("loader marquee scrolls a live strip of loaders", async ({ page }) => {
  await page.goto("/");
  const marquee = page.locator(".loader-marquee");
  await marquee.scrollIntoViewIfNeeded();

  await expect(marquee).toHaveAttribute(
    "data-marquee-cell-count",
    String(heroPack.styles.length * 2),
  );
  const cells = page.locator(".loader-marquee-half").first().locator(".loader-marquee-cell");
  await expect(cells).toHaveCount(heroPack.styles.length * 2);
  await expect(cells.first().locator(".loader-marquee-output")).not.toBeEmpty();

  // The strip itself moves via a CSS transform animation.
  const animationName = await page
    .locator(".loader-marquee-track")
    .evaluate((node) => getComputedStyle(node).animationName);
  expect(animationName).not.toBe("none");
});

test("reduced motion freezes the hero and the marquee", async ({ browser }) => {
  const context = await browser.newContext({ reducedMotion: "reduce" });
  const page = await context.newPage();
  await page.goto("/");

  const showcase = page.locator(".hero-showcase");
  await expect(page.locator(".hero-showcase-output")).not.toBeEmpty();
  const frame = await showcase.getAttribute("data-hero-frame");
  await page.waitForTimeout(450);
  await expect(showcase).toHaveAttribute("data-hero-frame", frame ?? "");

  const animationName = await page
    .locator(".loader-marquee-track")
    .evaluate((node) => getComputedStyle(node).animationName);
  expect(animationName).toBe("none");

  await context.close();
});

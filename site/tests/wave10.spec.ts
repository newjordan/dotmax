import { expect, test } from "@playwright/test";
import fs from "node:fs";

const heroPack = JSON.parse(
  fs.readFileSync(new URL("../public/catalog/hero.json", import.meta.url), "utf-8"),
) as { styles: Array<{ id: string }> };

test("page never scrolls horizontally", async ({ page }) => {
  await page.goto("/");
  const height = await page.evaluate(() => document.body.scrollHeight);
  for (let y = 0; y < height; y += 700) {
    await page.evaluate((top) => window.scrollTo(0, top), y);
    await page.waitForTimeout(60);
  }
  const overflow = await page.evaluate(
    () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
  );
  expect(overflow).toBeLessThanOrEqual(1);
});

test("hero controls step and shuffle the showcase, arrows work from the keyboard", async ({ page }) => {
  await page.goto("/");
  const showcase = page.locator(".hero-showcase");
  await expect(showcase).toHaveAttribute("data-hero-style", heroPack.styles[0].id);

  await page.getByRole("button", { name: "Next style" }).click();
  await expect(showcase).toHaveAttribute("data-hero-style", heroPack.styles[1].id);
  await page.getByRole("button", { name: "Previous style" }).click();
  await expect(showcase).toHaveAttribute("data-hero-style", heroPack.styles[0].id);

  // Hover/focus pauses the rotation and the progress bar.
  await expect(showcase).toHaveAttribute("data-hero-paused", "true");

  await page.keyboard.press("ArrowRight");
  await expect(showcase).toHaveAttribute("data-hero-style", heroPack.styles[1].id);

  const before = await showcase.getAttribute("data-hero-style");
  await page.getByRole("button", { name: "Random style" }).click();
  await expect(showcase).not.toHaveAttribute("data-hero-style", before ?? "");
});

test("mobile menu opens, links to sections, and closes", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== "mobile", "hamburger only renders below lg");
  await page.goto("/");
  await expect(page.locator("#mobile-nav")).toHaveCount(0);
  await page.getByRole("button", { name: "Open menu" }).click();
  const nav = page.locator("#mobile-nav");
  await expect(nav).toBeVisible();
  await expect(nav.locator(".mobile-nav-link")).toHaveCount(3);
  await nav.getByRole("link", { name: "Styles" }).click();
  await expect(page.locator("#mobile-nav")).toHaveCount(0);
});

test("collections, sort, and surprise-me narrow and open the catalog", async ({ page }) => {
  await page.goto("/");
  await page.locator("#loading-bars").scrollIntoViewIfNeeded();
  await expect(page.locator(".loading-bar-card")).toHaveCount(12);

  // Collections filter to a subset of bar themes.
  const retro = page.getByRole("button", { name: /Retro games/ });
  const retroCount = Number((await retro.locator("span").innerText()).trim());
  await retro.click();
  await expect(page.locator(".loading-bar-card")).toHaveCount(retroCount);
  const themes = await page.locator(".loading-bar-card").evaluateAll((nodes) =>
    Array.from(new Set(nodes.map((node) => node.getAttribute("data-theme")))),
  );
  for (const theme of themes) expect(["retro", "atari", "nintendo", "gameboy", "blocks"]).toContain(theme);

  // A–Z sort reorders the visible cards.
  await page.getByRole("button", { name: "A–Z" }).click();
  const names = await page.locator(".loading-bar-terminal-top span:first-child").allInnerTexts();
  expect(names).toEqual(names.slice().sort((a, b) => a.localeCompare(b)));

  // Toggling the collection off restores the curated default.
  await retro.click();
  await page.getByRole("button", { name: "Catalog" }).click();
  await expect(page.locator(".loading-bar-card")).toHaveCount(12);

  // Incremental expander adds a page without opening everything.
  await page.getByRole("button", { name: /Show 24 more/ }).click();
  await expect(page.locator(".loading-bar-card")).toHaveCount(36);

  // Surprise me opens a random style in the dialog.
  await page.getByRole("button", { name: "Surprise me" }).click();
  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await expect(dialog.locator(".style-dialog-position")).toHaveText(/\d+ \/ \d+/);
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
});

test("style dialog steps through the current filter with arrow keys and shows the palette", async ({ page }) => {
  await page.goto("/");
  await page.locator("#loading-bars").scrollIntoViewIfNeeded();
  await page.locator(".loading-bar-card").first().click();

  const dialog = page.getByRole("dialog", { name: "solid style" });
  await expect(dialog).toBeVisible();
  await expect(dialog.locator(".style-dialog-position")).toHaveText(/^1 \/ \d+$/);
  await expect(dialog.locator(".style-dialog-palette span[style*='background']").first()).toBeAttached();

  await page.keyboard.press("ArrowRight");
  await expect(page.getByRole("dialog")).toHaveAttribute("aria-label", "gradient style");
  await expect(page.locator(".style-dialog-position")).toHaveText(/^2 \/ \d+$/);
  await page.keyboard.press("ArrowLeft");
  await expect(page.getByRole("dialog")).toHaveAttribute("aria-label", "solid style");

  // Wrap-around: ← from the first entry lands on the last.
  await page.keyboard.press("ArrowLeft");
  const position = await page.locator(".style-dialog-position").innerText();
  const [current, total] = position.split("/").map((part) => Number(part.trim()));
  expect(current).toBe(total);

  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toHaveCount(0);
});

test("hero showcase responds to horizontal swipes on touch screens", async ({ page }, testInfo) => {
  test.skip(testInfo.project.name !== "mobile", "touch emulation only on the mobile project");
  await page.goto("/");
  const showcase = page.locator(".hero-showcase");
  await expect(showcase).toHaveAttribute("data-hero-style", heroPack.styles[0].id);
  const body = page.locator(".hero-showcase-body");
  await body.scrollIntoViewIfNeeded();
  const box = await body.boundingBox();
  if (!box) throw new Error("hero body not laid out");
  const y = box.y + box.height / 2;
  const swipe = async (fromX: number, toX: number) => {
    await body.dispatchEvent("touchstart", { touches: [{ identifier: 1, clientX: fromX, clientY: y }] });
    await body.dispatchEvent("touchend", { changedTouches: [{ identifier: 1, clientX: toX, clientY: y }] });
  };
  await swipe(box.x + box.width * 0.8, box.x + box.width * 0.2);
  await expect(showcase).toHaveAttribute("data-hero-style", heroPack.styles[1].id);
  await swipe(box.x + box.width * 0.2, box.x + box.width * 0.8);
  await expect(showcase).toHaveAttribute("data-hero-style", heroPack.styles[0].id);
  // A mostly-vertical drag (scrolling) must not change the style.
  await body.dispatchEvent("touchstart", { touches: [{ identifier: 2, clientX: box.x + 100, clientY: y }] });
  await body.dispatchEvent("touchend", { changedTouches: [{ identifier: 2, clientX: box.x + 130, clientY: y + 120 }] });
  await expect(showcase).toHaveAttribute("data-hero-style", heroPack.styles[0].id);
});

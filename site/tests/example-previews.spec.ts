import { expect, test } from "@playwright/test";
import { execFileSync } from "node:child_process";
import fs from "node:fs";

// Expected counts come from the committed catalog index, so regenerating the
// catalog (new themes/styles) never breaks these tests. The tests then assert
// the SERVED catalog matches the committed one.
// Residual constraint: `classic` must stay the first theme with `solid` as its
// first style — the dialog tests below open the first card by name.
const catalogIndex = JSON.parse(
  fs.readFileSync(new URL("../public/catalog/index.json", import.meta.url), "utf-8"),
) as {
  total: number;
  fps: number;
  frames_per_style: number;
  themes: Array<{ name: string; count: number; kind: string }>;
  styles: Array<{ kind: string }>;
};
const TOTAL = catalogIndex.total;
const THEME_PILL_COUNT = catalogIndex.themes.length + 1; // +1 for the "All" pill
const kindTotal = (kind: string) => catalogIndex.styles.filter((s) => s.kind === kind).length;

test("example cards render terminal previews without horizontal overflow", async ({ page }, testInfo) => {
  test.skip(true, "section parked behind SHOW_SECONDARY_SECTIONS");
  await page.goto("/");
  await page.locator("#examples").scrollIntoViewIfNeeded();

  const cards = page.locator(".example-card");
  const previews = page.locator(".example-terminal");
  const previewImages = page.locator(".example-terminal-image");
  const miniTerminals = page.locator(".mini-terminal");
  const collapsedCode = page.locator(".example-code-details:not([open])");
  const categoryThumbnails = page.locator(".category-menu-thumb img");

  await expect(cards).toHaveCount(15);
  await expect(previews).toHaveCount(15);
  await expect(previewImages).toHaveCount(8);
  await expect(miniTerminals).toHaveCount(7);
  await expect(collapsedCode).toHaveCount(15);
  await expect(categoryThumbnails).toHaveCount(4);

  const pageWidth = await page.evaluate(() => document.documentElement.clientWidth);
  const pageOverflow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
  expect(pageOverflow).toBeLessThanOrEqual(1);

  const previewBoxes = await previews.evaluateAll((nodes) =>
    nodes.map((node) => {
      const rect = node.getBoundingClientRect();
      return {
        left: rect.left,
        right: rect.right,
        height: rect.height,
      };
    }),
  );

  for (const box of previewBoxes) {
    expect(box.left).toBeGreaterThanOrEqual(0);
    expect(box.right).toBeLessThanOrEqual(pageWidth + 1);
    expect(box.height).toBeGreaterThan(220);
    expect(box.height).toBeLessThan(430);
  }

  const previewImageFits = await page.locator(".example-terminal-image, .filter-pill img").evaluateAll((nodes) =>
    nodes.every((node) => getComputedStyle(node).objectFit === "contain"),
  );
  expect(previewImageFits).toBe(true);

  await cards.first().scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`examples-${testInfo.project.name}.png`),
    fullPage: false,
  });
});

test("mini terminals load frame packs and advance visible animations", async ({ page }) => {
  test.skip(true, "section parked behind SHOW_SECONDARY_SECTIONS");
  await page.goto("/");
  await page.locator("#examples").scrollIntoViewIfNeeded();

  const terminals = page.locator(".mini-terminal");
  await expect(terminals).toHaveCount(7);

  for (let index = 0; index < 7; index += 1) {
    await terminals.nth(index).scrollIntoViewIfNeeded();
  }

  await expect(page.locator(".mini-terminal[data-loaded='true']")).toHaveCount(7);

  const nonEmptyCount = await terminals.evaluateAll((nodes) =>
    nodes.filter((node) => (node.textContent ?? "").replace(/\s/g, "").length > 0).length,
  );
  expect(nonEmptyCount).toBeGreaterThanOrEqual(6);

  const first = terminals.first();
  await first.scrollIntoViewIfNeeded();
  const initialFrame = await first.getAttribute("data-frame-index");
  await page.waitForTimeout(450);
  const nextFrame = await first.getAttribute("data-frame-index");
  expect(nextFrame).not.toBe(initialFrame);
});

test("reduced motion shows static mini terminal frames", async ({ browser }) => {
  test.skip(true, "section parked behind SHOW_SECONDARY_SECTIONS");
  const context = await browser.newContext({ reducedMotion: "reduce" });
  const page = await context.newPage();
  await page.goto("/");
  await page.locator("#examples").scrollIntoViewIfNeeded();

  const first = page.locator(".mini-terminal").first();
  await first.scrollIntoViewIfNeeded();
  await expect(first).toHaveAttribute("data-loaded", "true");
  await expect(first).toHaveAttribute("data-frame-index", "0");
  await page.waitForTimeout(450);
  await expect(first).toHaveAttribute("data-frame-index", "0");

  await context.close();
});

test("copy buttons expose copied feedback", async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: async (value: string) => {
          window.localStorage.setItem("copied-value", value);
        },
      },
    });
  });

  await page.goto("/");
  await page.locator("#patterns").scrollIntoViewIfNeeded();
  await page.getByRole("button", { name: "Open the pattern workbench" }).click();

  const firstCopy = page.locator("#patterns .copy-button").first();
  await expect(firstCopy).toHaveAttribute("data-copy-state", "idle");
  await firstCopy.click();
  await expect(firstCopy).toHaveAttribute("data-copy-state", "copied");
  await expect(firstCopy.locator(".copy-button-status")).toContainText("Copied");
  await expect(firstCopy).toHaveAccessibleName("Copied");
});

test("style library defaults to a curated set and expands to the full catalog", async ({ page }, testInfo) => {
  await page.goto("/");
  await page.locator("#loading-bars").scrollIntoViewIfNeeded();

  const grid = page.locator(".loading-bars-grid");
  await expect(grid).toHaveAttribute("data-loading-bar-count", String(TOTAL));
  // The catalog is gated: a curated 12 render by default to keep the page scannable.
  await expect(page.locator(".loading-bar-card")).toHaveCount(12);
  await expect(page.locator(".loading-theme-pill")).toHaveCount(THEME_PILL_COUNT);
  await expect(page.locator(".loading-kind-tab")).toHaveCount(6);

  // Expander reveals the full catalog on demand.
  await page.getByRole("button", { name: new RegExp(`Browse all ${TOTAL} styles`) }).click();
  await expect(page.locator(".loading-bar-card")).toHaveCount(TOTAL);

  // Catalog v2: colored frames at 12 fps, split per theme, with style source.
  // Relative fetches resolve against the page URL, so they work under any base.
  const catalogShape = await page.evaluate(async () => {
    const index = await (await fetch("catalog/index.json")).json();
    const pack = await (await fetch("catalog/themes/classic.json")).json();
    type Frame = { t: string[]; c?: string[] };
    type Style = { frames: Frame[]; source: string; palette: string[] };
    const styles = pack.styles as Style[];
    return {
      fps: index.fps,
      framesPerStyle: index.frames_per_style,
      total: index.total,
      themeCount: index.themes.length,
      colored: styles.some((s) => s.frames.some((f) => f.c)),
      allHaveSource: styles.every((s) => s.source.includes("impl ProgressStyle")),
    };
  });
  expect(catalogShape.fps).toBe(12);
  expect(catalogShape.framesPerStyle).toBe(48);
  expect(catalogShape.total).toBe(TOTAL);
  expect(catalogShape.themeCount).toBe(catalogIndex.themes.length);
  expect(catalogShape.colored).toBe(true);
  expect(catalogShape.allHaveSource).toBe(true);

  const firstCard = page.locator(".loading-bar-card").first();
  await expect(firstCard.locator(".loading-bar-output")).not.toBeEmpty();
  const initialFrame = await grid.getAttribute("data-loading-bar-frame");
  await page.waitForTimeout(350);
  await expect(grid).not.toHaveAttribute("data-loading-bar-frame", initialFrame ?? "");

  // Color actually reaches the DOM as inline spans once packs load.
  await expect(page.locator(".loading-bar-output span[style*='color']").first()).toBeAttached();

  await page.getByPlaceholder("Search wildlife, quantum, sinewave, spinner...").fill("quantum");
  const filteredCount = await page.locator(".loading-bar-card").count();
  expect(filteredCount).toBeGreaterThan(0);
  expect(filteredCount).toBeLessThan(TOTAL);

  // Kind tabs gate the non-bar families.
  await page.getByPlaceholder("Search wildlife, quantum, sinewave, spinner...").fill("");
  await page.getByRole("tab", { name: /Borders/ }).click();
  await expect(page.locator(".loading-bar-card")).toHaveCount(kindTotal("border"));
  await page.getByRole("tab", { name: /Spinners/ }).click();
  await expect(page.locator(".loading-bar-card")).toHaveCount(kindTotal("spinner"));
  await expect(page.locator(".loading-bar-output span[style*='color']").first()).toBeAttached();

  await page.locator(".loading-bar-card").first().scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`style-library-${testInfo.project.name}.png`),
    fullPage: false,
  });
});

test("style cards open a detail dialog with three copy formats", async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: async (value: string) => {
          window.localStorage.setItem("copied-value", value);
        },
      },
    });
  });

  await page.goto("/");
  await page.locator("#loading-bars").scrollIntoViewIfNeeded();
  await page.locator(".loading-bar-card").first().click();

  const dialog = page.getByRole("dialog", { name: "solid style" });
  await expect(dialog).toBeVisible();
  await expect(dialog.locator(".style-dialog-tab")).toHaveCount(4);

  // Default tab: dotmax crate snippet with the exact style lookup.
  await expect(dialog.locator(".style-dialog-code")).toContainText('styles_for_theme("classic")');
  await expect(dialog.locator(".style-dialog-code")).toContainText('s.name() == "solid"');

  // Standalone tab: a full single-file program defaulting to this style.
  await dialog.getByRole("tab", { name: "standalone .rs" }).click();
  await expect(dialog.locator(".style-dialog-code")).toContainText('const DEFAULT_STYLE: &str = "solid";');
  await expect(dialog.locator(".style-dialog-code")).toContainText("fn main()");

  // Shell tab: zero-dependency ANSI playback script.
  await dialog.getByRole("tab", { name: "shell script" }).click();
  await expect(dialog.locator(".style-dialog-code")).toContainText("#!/usr/bin/env bash");
  await expect(dialog.locator(".style-dialog-code")).toContainText("frames=(");

  // Source tab: the style's Rust implementation, for reading.
  await dialog.getByRole("tab", { name: "style source" }).click();
  await expect(dialog.locator(".style-dialog-code")).toContainText("impl ProgressStyle for");

  // Copy gives feedback and lands the content.
  const copy = dialog.locator(".copy-button");
  await copy.click();
  await expect(copy).toHaveAttribute("data-copy-state", "copied");
  const copied = await page.evaluate(() => window.localStorage.getItem("copied-value"));
  expect(copied).toContain("impl ProgressStyle for");

  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);
});

test("copied shell script actually plays ANSI frames in bash", async ({ page }, testInfo) => {
  await page.goto("/");
  await page.locator("#loading-bars").scrollIntoViewIfNeeded();
  await page.locator(".loading-bar-card").first().click();

  const dialog = page.getByRole("dialog", { name: "solid style" });
  await dialog.getByRole("tab", { name: "shell script" }).click();
  await expect(dialog.locator(".style-dialog-code")).toContainText("#!/usr/bin/env bash");
  const script = await dialog.locator(".style-dialog-code").innerText();

  const scriptPath = testInfo.outputPath("style.sh");
  fs.writeFileSync(scriptPath, script);
  let stdout = "";
  try {
    stdout = execFileSync("bash", [scriptPath], { timeout: 1500, encoding: "utf-8" });
  } catch (error) {
    // The script loops forever by design; the timeout kill is expected.
    stdout = String((error as { stdout?: string }).stdout ?? "");
  }
  // Frames painted: ANSI truecolor runs, cursor-up rewinds, and visible glyphs.
  expect(stdout).toContain("[38;2;");
  expect(stdout).toContain("[4A");
  expect(stdout.replace(/\[[0-9;?]*[A-Za-z]/g, "").trim().length).toBeGreaterThan(0);

  await page.screenshot({
    path: testInfo.outputPath(`style-dialog-${testInfo.project.name}.png`),
    fullPage: false,
  });
});

test("TUI pattern workbench exposes tables folders tabs and schematics", async ({ page }) => {
  await page.goto("/");
  await page.locator("#patterns").scrollIntoViewIfNeeded();

  // The workbench is collapsed by default; the resource index stays visible.
  await expect(page.locator(".pattern-resource-card")).toHaveCount(8);
  await expect(page.locator(".pattern-resource-card").first()).toContainText("Columns, filters");
  await page.getByRole("button", { name: "Open the pattern workbench" }).click();

  await expect(page.locator(".pattern-tab")).toHaveCount(8);
  await page.getByRole("button", { name: "Open Schematic resources" }).click();
  await expect(page.locator(".schematic-recipe-row")).toHaveCount(5);
  await page.getByRole("button", { name: "Open Research table resources" }).click();
  await expect(page.locator(".resource-table tbody tr")).toHaveCount(8);
  await expect(page.locator(".table-recipe-row")).toHaveCount(5);
  await expect(page.locator(".table-recipe-detail")).toContainText("Column schema");
  await page.getByRole("button", { name: /faceted-filter-bar/ }).click();
  await expect(page.locator(".table-recipe-code")).toContainText("struct FilterState");
  await expect(page.locator(".table-recipe-preview")).toContainText("visible  42 / 586 rows");
  await page.getByPlaceholder("Search table, folders, schematic, inspector...").fill("folder");
  await expect(page.locator(".resource-table tbody tr")).toHaveCount(1);

  await page.getByRole("button", { name: "Folders", exact: true }).click();
  await expect(page.locator(".folder-node")).toHaveCount(5);
  await page.getByRole("button", { name: /src\/views\/schematic.rs/ }).click();
  await expect(page.locator(".folder-detail-panel")).toContainText("stable node ids");
  await expect(page.locator(".folder-browser-row")).toHaveCount(4);
  await expect(page.locator(".folder-browser-detail")).toContainText("Arena tree state");
  await page.getByRole("button", { name: /preview-cache/ }).click();
  await expect(page.locator(".folder-browser-code")).toContainText("enum PreviewModel");
  await expect(page.locator(".folder-browser-preview")).toContainText("dotmax thumb");

  await page.getByRole("button", { name: "Schematic", exact: true }).click();
  await expect(page.locator(".schematic-node")).toHaveCount(6);
  await expect(page.locator(".schematic-notes")).toContainText("dotmax owns dense visual previews");
  await expect(page.locator(".schematic-recipe-row")).toHaveCount(5);
  await expect(page.locator(".schematic-recipe-detail")).toContainText("Stable node model");
  await page.getByRole("button", { name: /health-propagation/ }).click();
  await expect(page.locator(".schematic-recipe-code")).toContainText("fn derived_health");
  await expect(page.locator(".schematic-recipe-preview")).toContainText("export blocked");

  await page.getByRole("button", { name: "Tabs", exact: true }).click();
  await expect(page.locator(".tab-recipe-row")).toHaveCount(4);
  await expect(page.locator(".tab-strip-preview span")).toHaveCount(5);
  await expect(page.locator(".tab-recipe-detail")).toContainText("Typed view router");
  await page.getByRole("button", { name: /preview-backed-tabs/ }).click();
  await expect(page.locator(".tab-recipe-code")).toContainText("struct TabSummary");

  await page.getByRole("button", { name: "Blueprints", exact: true }).click();
  await expect(page.locator(".blueprint-card")).toHaveCount(5);
  await expect(page.locator(".blueprint-card").first()).toContainText("struct TableState");
  await expect(page.locator(".blueprint-card").first()).toContainText("rows: Vec<RowModel>");

  await page.getByRole("button", { name: "Resource kits", exact: true }).click();
  await expect(page.locator(".kit-card")).toHaveCount(4);
  await expect(page.locator(".kit-card").first()).toContainText("Research dashboard");
  await expect(page.locator(".kit-card").first()).toContainText("cargo add ratatui crossterm");
  await expect(page.locator(".kit-card").nth(2)).toContainText("petgraph");

  await page.getByRole("button", { name: "Contracts", exact: true }).click();
  await expect(page.locator(".contract-row")).toHaveCount(6);
  await expect(page.locator(".contract-detail")).toContainText("table.move-selection");
  await page.getByRole("button", { name: /schematic.focus-node/ }).click();
  await expect(page.locator(".contract-detail")).toContainText("focus only lands on visible nodes");
  await expect(page.locator(".contract-snippet")).toContainText("Command::FocusNode");

  await page.getByRole("button", { name: "Layouts", exact: true }).click();
  await expect(page.locator(".layout-card")).toHaveCount(4);
  await expect(page.locator(".layout-wireframe span")).toHaveCount(15);
  await expect(page.locator(".layout-card").first()).toContainText("Table + inspector shell");
  await expect(page.locator(".layout-code").first()).toContainText("Layout::vertical");
});

test("supporting cards stay visual instead of text-only", async ({ page }, testInfo) => {
  await page.goto("/");

  await expect(page.locator(".feature-card-visual img")).toHaveCount(6);
  await expect(page.locator(".doc-card-visual img")).toHaveCount(5);
  await expect(page.locator(".open-source-visual img")).toHaveCount(1);

  // .gallery-item img dropped: the gallery is parked behind SHOW_SECONDARY_SECTIONS.
  const containedImages = await page.locator(".feature-card-visual img, .doc-card-visual img, .open-source-visual img").evaluateAll((nodes) =>
    nodes.every((node) => getComputedStyle(node).objectFit === "contain"),
  );
  expect(containedImages).toBe(true);

  await page.locator(".feature-card").first().scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`supporting-cards-${testInfo.project.name}.png`),
    fullPage: false,
  });
});

test("install docs and open-source sections are visually led", async ({ page }, testInfo) => {
  test.skip(true, "section parked behind SHOW_SECONDARY_SECTIONS");
  await page.goto("/");

  await page.locator("#install").scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`install-${testInfo.project.name}.png`),
    fullPage: false,
  });

  await page.locator("#docs").scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`docs-open-source-${testInfo.project.name}.png`),
    fullPage: false,
  });

  await page.locator(".open-source-band").scrollIntoViewIfNeeded();
  await page.screenshot({
    path: testInfo.outputPath(`open-source-${testInfo.project.name}.png`),
    fullPage: false,
  });
});

test("command palette opens with Cmd/Ctrl+K and searches the catalog", async ({ page }) => {
  await page.goto("/");

  // Trigger via the nav button.
  await page.getByRole("button", { name: "Open command menu" }).click();
  const dialog = page.getByRole("dialog", { name: "Command menu" });
  await expect(dialog).toBeVisible();

  // Static quick actions show before typing.
  await expect(page.locator(".cmdk-item")).not.toHaveCount(0);

  // Fuzzy search reaches into the full style catalog.
  await page.getByPlaceholder("Search examples, loading bars, docs…").fill("gradient");
  await expect(page.locator(".cmdk-group-label", { hasText: "Styles" })).toBeVisible();

  // Escape closes it.
  await page.keyboard.press("Escape");
  await expect(dialog).toHaveCount(0);

  // Keyboard shortcut re-opens it.
  await page.keyboard.press("ControlOrMeta+k");
  await expect(page.getByRole("dialog", { name: "Command menu" })).toBeVisible();
});

test("build-with-ai section exposes agent prompt and llms.txt", async ({ page }) => {
  await page.goto("/");
  await page.locator("#build-with-ai").scrollIntoViewIfNeeded();

  await expect(page.getByRole("heading", { name: "Built to be built by agents." })).toBeVisible();
  await expect(page.getByRole("button", { name: "Copy prompt" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Open llms.txt" })).toHaveAttribute("href", "/llms.txt");
  // Roadmap items are labeled honestly, not claimed as shipped.
  await expect(page.locator(".ai-roadmap")).toContainText("roadmap");
});

test("llms.txt is served at the site root", async ({ page }) => {
  const response = await page.request.get("llms.txt");
  expect(response.ok()).toBe(true);
  const body = await response.text();
  expect(body).toContain("# dotmax");
  expect(body).toContain("cargo add dotmax");
  expect(body).toContain("For AI agents");
});
